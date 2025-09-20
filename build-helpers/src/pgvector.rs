use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build pgvector extension with PostgreSQL headers and binaries
pub fn build(
    target_dir: &Path,
    target: &str,
    source_path: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Default source path if not provided
    let default_path;
    let pgvector_dir = if let Some(path) = source_path {
        path
    } else {
        default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("../src-databases/pgvector");
        &default_path
    };

    if !pgvector_dir.exists() {
        return Err(format!(
            "pgvector source directory not found at: {}",
            pgvector_dir.display()
        )
        .into());
    }

    // Use dedicated pgvector directory
    let pgvector_build_dir = target_dir.join("pgvector");
    fs::create_dir_all(&pgvector_build_dir)?;

    // Download and setup PostgreSQL binaries in pgvector source directory
    let postgres_dir = setup_postgresql_binaries(pgvector_dir, target)?;

    // Expected shared library filename based on platform
    let library_name = if target.contains("windows") {
        "vector.dll"
    } else if target.contains("darwin") {
        "vector.dylib"
    } else {
        "vector.so"
    };

    let target_library_path = pgvector_build_dir.join(library_name);

    // Always build for testing - comment out the skip condition
    // if target_library_path.exists() {
    //     println!(
    //         "pgvector library already exists at {:?}",
    //         target_library_path
    //     );
    //     return Ok(target_library_path);
    // }

    // Create target directory for built files
    fs::create_dir_all(&pgvector_build_dir)?;

    // Build pgvector extension in source directory
    build_pgvector_extension(pgvector_dir, &postgres_dir, target)?;

    // Copy built library file and control file to target directory
    let source_library_path = pgvector_dir.join(library_name);
    if source_library_path.exists() {
        fs::copy(&source_library_path, &target_library_path)?;
    } else {
        return Err(format!(
            "pgvector library not found after build: {}",
            source_library_path.display()
        )
        .into());
    }

    // Copy vector.control file to target directory
    let source_control = pgvector_dir.join("vector.control");
    let target_control = pgvector_build_dir.join("vector.control");
    if source_control.exists() {
        fs::copy(&source_control, &target_control)?;
    } else {
        return Err("pgvector control file not found".into());
    }

    // Copy SQL files to target directory
    let source_sql_dir = pgvector_dir.join("sql");
    let target_sql_dir = pgvector_build_dir.join("sql");
    if source_sql_dir.exists() {
        fs::create_dir_all(&target_sql_dir)?;
        for entry in fs::read_dir(&source_sql_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                let target_path = target_sql_dir.join(path.file_name().unwrap());
                fs::copy(&path, &target_path)?;
            }
        }
    } else {
        return Err("pgvector SQL directory not found".into());
    }

    // Verify the built library exists
    if target_library_path.exists() {
        Ok(target_library_path)
    } else {
        Err("pgvector extension library was not built successfully".into())
    }
}

/// Download and setup PostgreSQL binaries for the target platform
fn setup_postgresql_binaries(
    pgvector_dir: &Path,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let postgres_dir = pgvector_dir.join("postgresql-17.5.0");

    // Skip download if already exists
    if postgres_dir.exists() {
        println!("PostgreSQL binaries already exist at {:?}", postgres_dir);
        return Ok(postgres_dir);
    }

    println!("Setting up PostgreSQL binaries for pgvector build...");

    // Determine the correct PostgreSQL binary package for the target
    let postgres_package = match target {
        t if t.contains("aarch64") && t.contains("apple") => {
            "postgresql-17.5.0-aarch64-apple-darwin.tar.gz"
        }
        t if t.contains("x86_64") && t.contains("apple") => {
            "postgresql-17.5.0-x86_64-apple-darwin.tar.gz"
        }
        t if t.contains("x86_64") && t.contains("linux") => {
            "postgresql-17.5.0-x86_64-unknown-linux-gnu.tar.gz"
        }
        t if t.contains("aarch64") && t.contains("linux") => {
            "postgresql-17.5.0-aarch64-unknown-linux-gnu.tar.gz"
        }
        t if t.contains("x86_64") && t.contains("windows") => {
            "postgresql-17.5.0-x86_64-pc-windows-msvc.zip"
        }
        _ => {
            println!(
                "Warning: Unsupported target platform for PostgreSQL binaries: {}",
                target
            );
            return Ok(postgres_dir); // Return empty directory for unsupported platforms
        }
    };

    let download_url = format!(
        "https://github.com/theseus-rs/postgresql-binaries/releases/download/17.5.0/{}",
        postgres_package
    );

    println!("Downloading PostgreSQL binaries: {}", download_url);

    // Download the PostgreSQL binaries to pgvector directory
    let archive_path = pgvector_dir.join(postgres_package);
    download_file(&download_url, &archive_path)?;

    // Extract the archive directly to postgresql-17.5.0 (removing platform-specific naming)
    extract_archive(&archive_path, &postgres_dir)?;

    // Clean up the archive
    fs::remove_file(&archive_path).ok();

    println!("PostgreSQL binaries extracted to {:?}", postgres_dir);
    Ok(postgres_dir)
}

/// Build the pgvector extension using make
fn build_pgvector_extension(
    pgvector_dir: &Path,
    postgres_dir: &Path,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up environment for PostgreSQL
    let pg_config_path = if target.contains("windows") {
        postgres_dir.join("bin").join("pg_config.exe")
    } else {
        postgres_dir.join("bin").join("pg_config")
    };

    // Determine make command
    let make_cmd = if target.contains("windows") {
        "nmake"
    } else {
        "make"
    };

    // Build the extension
    let mut cmd = Command::new(make_cmd);
    cmd.current_dir(pgvector_dir)
        .env("PG_CONFIG", &pg_config_path);

    // Add platform-specific flags
    if target.contains("darwin") {
        if target.contains("aarch64") || target.contains("arm64") {
            // ARM64 macOS - disable march=native for portability and fix debug flag issue
            cmd.env("OPTFLAGS", "");
        }
    } else if target.contains("windows") {
        // Windows-specific makefile
        cmd.args(&["/f", "Makefile.win"]);
        // Set PGROOT required by Makefile.win with proper Windows path format
        // Remove UNC prefix \\?\ that display() might add, as MSVC can't handle it
        let pgroot_path = postgres_dir.display().to_string();
        let pgroot_clean = if pgroot_path.starts_with(r"\\?\") {
            pgroot_path[4..].to_string()
        } else {
            pgroot_path
        };
        cmd.env("PGROOT", pgroot_clean);
    } else if target.contains("powerpc") || target.contains("ppc64") {
        // PowerPC doesn't support march=native
        cmd.env("OPTFLAGS", "");
    }

    // Override potentially problematic PostgreSQL build flags
    // The 'debug' flag might be coming from PostgreSQL's build configuration
    cmd.env("enable_debug", "no");
    cmd.env("ENABLE_DEBUG", "no");
    cmd.env("DEBUG", "");
    cmd.env("PROFILE", "");

    // Get correct SDK path on macOS
    if target.contains("darwin") {
        // Use xcrun to get the correct SDK path
        if let Ok(sdk_output) = Command::new("xcrun")
            .args(&["--sdk", "macosx", "--show-sdk-path"])
            .output()
        {
            if sdk_output.status.success() {
                let sdk_path = String::from_utf8_lossy(&sdk_output.stdout)
                    .trim()
                    .to_string();

                // Create a wrapper script for pg_config to fix the SDK path
                let wrapper_dir = pgvector_dir.join("pg_config_wrapper");
                fs::create_dir_all(&wrapper_dir)?;
                let wrapper_script = wrapper_dir.join("pg_config");
                let original_pg_config = pg_config_path.display();

                let wrapper_content = format!(
                    r#"#!/bin/bash
# Wrapper script to fix PostgreSQL SDK path
case "$1" in
    --cppflags)
        echo "-isysroot {} -I/opt/homebrew/opt/icu4c/include -I/opt/homebrew/opt/openssl/include"
        ;;
    --cflags)
        "{}" "$@" | sed 's|-isysroot /Library/Developer/CommandLineTools/SDKs/MacOSX[0-9]*\.[0-9]*\.sdk|-isysroot {}|g'
        ;;
    *)
        # For all other flags, run original pg_config and fix any SDK paths in the output
        "{}" "$@" | sed 's|/Library/Developer/CommandLineTools/SDKs/MacOSX[0-9]*\.[0-9]*\.sdk|{}|g'
        ;;
esac
"#,
                    sdk_path, original_pg_config, sdk_path, original_pg_config, sdk_path
                );

                fs::write(&wrapper_script, wrapper_content)?;

                // Make the wrapper executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&wrapper_script)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&wrapper_script, perms)?;
                }

                // Use the wrapper script instead of the original pg_config
                cmd.env("PG_CONFIG", &wrapper_script);
                cmd.env(
                    "PATH",
                    format!(
                        "{}:{}",
                        wrapper_dir.display(),
                        std::env::var("PATH").unwrap_or_default()
                    ),
                );

                // Use PostgreSQL's official environment variables to override flags
                // PG_CPPFLAGS will be prepended to CPPFLAGS (overriding the hardcoded SDK path)
                cmd.env("PG_CPPFLAGS", format!("-isysroot {}", sdk_path));
                cmd.env("PG_CFLAGS", format!("-isysroot {}", sdk_path));
                cmd.env("PG_LDFLAGS", format!("-isysroot {}", sdk_path));

                // Also patch the PostgreSQL Makefile.global to fix hardcoded PG_SYSROOT
                let makefile_global =
                    pgvector_dir.join("postgresql-17.5.0/lib/pgxs/src/Makefile.global");
                if makefile_global.exists() {
                    // Get the current wrong SDK path from pg_config --cppflags
                    if let Ok(cppflags_output) =
                        Command::new(&pg_config_path).arg("--cppflags").output()
                    {
                        if cppflags_output.status.success() {
                            let cppflags_str = String::from_utf8_lossy(&cppflags_output.stdout);
                            // Extract the wrong SDK path using regex or string matching
                            if let Some(start) = cppflags_str.find("-isysroot ") {
                                let remaining = &cppflags_str[start + 10..]; // Skip "-isysroot "
                                if let Some(end) = remaining.find(" ") {
                                    let wrong_sdk_path = &remaining[..end];

                                    // Now patch the Makefile.global
                                    let content = fs::read_to_string(&makefile_global)?;
                                    let fixed_content = content.replace(
                                        &format!("PG_SYSROOT = {}", wrong_sdk_path),
                                        &format!("PG_SYSROOT = {}", sdk_path),
                                    );
                                    fs::write(&makefile_global, fixed_content)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let output = cmd.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        eprintln!(
            "Error: pgvector extension build failed with exit code: {:?}",
            output.status.code()
        );
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        return Err("pgvector extension build failed".into());
    }
    Ok(())
}

/// Download a file from URL to local path
fn download_file(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let response = ureq::get(url).call()?;
    let mut file = fs::File::create(path)?;

    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer)?;
    file.write_all(&buffer)?;

    Ok(())
}

/// Extract tar.gz or zip archive, removing platform-specific top-level directories
fn extract_archive(
    archive_path: &Path,
    extract_to: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(extract_to)?;

    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        // Extract ZIP file
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Find the common prefix (platform-specific directory name)
        let mut common_prefix: Option<String> = None;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            if let Some(path) = file.enclosed_name() {
                let path_str = path.to_string_lossy();
                if let Some(first_component) = path_str.split('/').next() {
                    if common_prefix.is_none() {
                        common_prefix = Some(first_component.to_string());
                    }
                }
            }
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };

            // Remove the common prefix from the path
            let relative_path = if let Some(ref prefix) = common_prefix {
                file_path.strip_prefix(prefix).unwrap_or(file_path)
            } else {
                file_path
            };

            let outpath = extract_to.join(relative_path);

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Make executable on Unix
            #[cfg(unix)]
            if outpath.file_name().and_then(|s| s.to_str()) == Some("pg_config") {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&outpath)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&outpath, perms)?;
            }
        }
    } else {
        // Extract tar.gz file to temporary location first
        let temp_dir = extract_to.parent().unwrap().join("temp_postgres_extract");
        fs::create_dir_all(&temp_dir)?;

        let tar_gz = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&temp_dir)?;

        // Find the platform-specific directory and move its contents
        for entry in fs::read_dir(&temp_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                // Move contents from platform-specific dir to target dir
                move_dir_contents(&entry.path(), extract_to)?;
                break;
            }
        }

        // Clean up temporary directory
        fs::remove_dir_all(&temp_dir).ok();

        // Make pg_config executable on Unix
        #[cfg(unix)]
        {
            let pg_config_path = extract_to.join("bin").join("pg_config");
            if pg_config_path.exists() {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&pg_config_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&pg_config_path, perms)?;
            }
        }
    }

    Ok(())
}

/// Move contents from source directory to destination directory
fn move_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            move_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
