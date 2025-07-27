use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const GIT_LFS_VERSION: &str = "v3.7.0";

fn download_git_lfs(url: &str, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading git-lfs from: {}", url);

    let response = ureq::get(url).call()?;
    let mut reader = response.into_reader();

    let mut file = fs::File::create(target_path)?;
    std::io::copy(&mut reader, &mut file)?;

    Ok(())
}

fn extract_git_lfs(
    archive_path: &Path,
    target_dir: &Path,
    is_zip: bool,
    target_binary_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;

    if is_zip {
        // Extract ZIP file
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let filename = file.name();

            if filename.ends_with("git-lfs") || filename.ends_with("git-lfs.exe") {
                let output_path = target_dir.join(target_binary_name);

                let mut outfile = fs::File::create(&output_path)?;
                std::io::copy(&mut file, &mut outfile)?;

                return Ok(output_path);
            }
        }
    } else {
        // Extract tar.gz file
        let tar_gz = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if path.file_name() == Some(std::ffi::OsStr::new("git-lfs")) {
                let output_path = target_dir.join(target_binary_name);
                entry.unpack(&output_path)?;
                return Ok(output_path);
            }
        }
    }

    Err("git-lfs binary not found in archive".into())
}

fn create_binary_symlink(
    source_dir: &Path,
    target_dir: &Path,
    full_binary_name: &str,
    simple_name: &str,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_binary_path = source_dir.join(full_binary_name);
    let target_binary_path = target_dir.join(simple_name);

    // Remove existing symlink/shortcut if it exists
    if target_binary_path.exists() {
        fs::remove_file(&target_binary_path)?;
    }

    // Ensure target directory exists
    fs::create_dir_all(target_dir)?;

    if target.contains("windows") {
        // Windows: Create a batch file that calls the actual binary
        let batch_content = format!("@echo off\n\"{}\" %*", source_binary_path.to_string_lossy());
        let batch_path = target_dir.join(format!("{}.bat", simple_name));
        fs::write(&batch_path, batch_content)?;
        println!("Created Windows batch file: {:?}", batch_path);
    } else {
        // Unix: Create symlink
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&source_binary_path, &target_binary_path)?;
            println!(
                "Created symlink: {:?} -> {:?}",
                target_binary_path, source_binary_path
            );
        }
    }

    Ok(())
}

fn build_mistralrs_server(
    target_dir: &Path,
    target: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    println!("Building mistralrs-server...");

    let mistralrs_dir = Path::new("mistralrs-server");

    // Always build mistralrs-server in release mode
    let binary_name = if target.contains("windows") {
        format!("mistralrs-server-{}.exe", target)
    } else {
        format!("mistralrs-server-{}", target)
    };

    // Use dedicated mistralrs-build directory
    let mistralrs_build_dir = target_dir.join("mistralrs-build");
    fs::create_dir_all(&mistralrs_build_dir)?;
    let target_path = mistralrs_build_dir.join(&binary_name);

    // Skip build if binary already exists
    if target_path.exists() {
        println!(
            "mistralrs-server binary already exists at {:?}",
            target_path
        );
        return Ok(target_path);
    }

    // Build the mistralrs-server with appropriate features based on platform
    let mut cmd = Command::new("cargo");

    cmd.arg("build")
        .arg("--manifest-path")
        .arg(mistralrs_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&mistralrs_build_dir)
        .arg("--release"); // Always build in release mode

    // Add platform-specific features
    if target.contains("darwin") {
        // macOS: use metal acceleration
        cmd.arg("--features").arg("metal,accelerate");
    } else if target.contains("linux") && env::var("CUDA_PATH").is_ok() {
        // Linux with CUDA if available
        cmd.arg("--features").arg("cuda,cudnn,flash-attn");
    } else if target.contains("windows") && env::var("CUDA_PATH").is_ok() {
        // Windows: use DirectML
        cmd.arg("--features").arg("cuda,cudnn,flash-attn");
    } else {
        //do nothing for unsupported platforms
    }

    println!("Running: {:?}", cmd);
    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("Failed to build mistralrs-server:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        return Err("Failed to build mistralrs-server".into());
    }

    // Find the built binary (cargo always outputs mistralrs-server or mistralrs-server.exe)
    let original_binary_name = if target.contains("windows") {
        "mistralrs-server.exe"
    } else {
        "mistralrs-server"
    };
    let built_binary = mistralrs_build_dir
        .join("release")
        .join(original_binary_name);

    if !built_binary.exists() {
        return Err(format!("Built binary not found at {:?}", built_binary).into());
    }

    // Copy and rename to mistralrs-build directory with target triple
    fs::copy(&built_binary, &target_path)?;

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }

    println!("Successfully built mistralrs-server to {:?}", target_path);
    Ok(target_path)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Determine the target platform
    let target = env::var("TARGET").unwrap();

    let (platform, arch, extension) = if target.contains("windows") {
        if target.contains("x86_64") {
            ("windows", "amd64", "zip")
        } else if target.contains("aarch64") {
            ("windows", "arm64", "zip")
        } else {
            panic!("Unsupported Windows architecture: {}", target);
        }
    } else if target.contains("darwin") {
        if target.contains("x86_64") {
            ("darwin", "amd64", "zip")
        } else if target.contains("aarch64") {
            ("darwin", "arm64", "zip")
        } else {
            panic!("Unsupported macOS architecture: {}", target);
        }
    } else if target.contains("linux") {
        if target.contains("x86_64") {
            ("linux", "amd64", "tar.gz")
        } else if target.contains("aarch64") {
            ("linux", "arm64", "tar.gz")
        } else {
            panic!("Unsupported Linux architecture: {}", target);
        }
    } else {
        panic!("Unsupported platform: {}", target);
    };

    // Get the output directory and build profile
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // OUT_DIR is typically target/{profile}/build/{package}/out
    // We want to get to target/
    let target_dir = Path::new(&out_dir)
        .parent() // removes /out
        .unwrap()
        .parent() // removes /{package}
        .unwrap()
        .parent() // removes /build
        .unwrap()
        .parent() // removes /{profile}
        .unwrap();

    // Use dedicated git-lfs directory
    let git_lfs_dir = target_dir.join("git-lfs");
    fs::create_dir_all(&git_lfs_dir)
        .unwrap_or_else(|e| panic!("Failed to create git-lfs directory: {}", e));

    // Use target triple format for binary naming
    let target_binary_name = if target.contains("windows") {
        format!("git-lfs-{}.exe", target)
    } else {
        format!("git-lfs-{}", target)
    };

    let target_path = git_lfs_dir.join(&target_binary_name);

    // Skip download if binary already exists
    if target_path.exists() {
        println!("git-lfs binary already exists at {:?}", target_path);
    } else {
        // Create a temporary directory for download
        let temp_dir = Path::new(&out_dir).join("git-lfs-download");
        fs::create_dir_all(&temp_dir).unwrap();

        // Construct the download URL and filename
        let archive_name = format!(
            "git-lfs-{}-{}-{}.{}",
            platform, arch, GIT_LFS_VERSION, extension
        );
        let download_url = format!(
            "https://github.com/git-lfs/git-lfs/releases/download/{}/{}",
            GIT_LFS_VERSION, archive_name
        );

        let archive_path = temp_dir.join(&archive_name);

        // Download the archive
        if let Err(e) = download_git_lfs(&download_url, &archive_path) {
            panic!("Failed to download git-lfs: {}", e);
        }

        // Extract the binary
        let extracted_path = extract_git_lfs(
            &archive_path,
            &temp_dir,
            extension == "zip",
            &target_binary_name,
        )
        .unwrap_or_else(|e| panic!("Failed to extract git-lfs: {}", e));

        // Copy to target directory
        fs::copy(&extracted_path, &target_path)
            .unwrap_or_else(|e| panic!("Failed to copy git-lfs binary: {}", e));

        // Clean up temporary files
        fs::remove_dir_all(&temp_dir).ok();

        println!("Successfully installed git-lfs to {:?}", target_path);
    }

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms).unwrap();
    }

    // Build mistralrs-server
    println!("cargo:rerun-if-changed=mistralrs-server");
    let mistralrs_path = match build_mistralrs_server(&target_dir, &target) {
        Ok(path) => Some(path),
        Err(e) => {
            eprintln!("Warning: Failed to build mistralrs-server: {}", e);
            eprintln!("Continuing without mistralrs-server binary");
            None
        }
    };

    // Create symlinks in target/{profile} directories for both debug and release
    for build_profile in ["debug", "release"] {
        let profile_dir = target_dir.join(build_profile);
        fs::create_dir_all(&profile_dir).ok();

        // Create git-lfs symlink
        if let Err(e) = create_binary_symlink(
            &git_lfs_dir,
            &profile_dir,
            &target_binary_name,
            "git-lfs",
            &target,
        ) {
            eprintln!(
                "Warning: Failed to create git-lfs symlink in {} directory: {}",
                build_profile, e
            );
        }

        // Create mistralrs-server symlink if it was built successfully
        if let Some(ref mistralrs_binary_path) = mistralrs_path {
            let mistralrs_binary_name =
                mistralrs_binary_path.file_name().unwrap().to_str().unwrap();
            let mistralrs_source_dir = mistralrs_binary_path.parent().unwrap();
            if let Err(e) = create_binary_symlink(
                mistralrs_source_dir,
                &profile_dir,
                mistralrs_binary_name,
                "mistralrs-server",
                &target,
            ) {
                eprintln!(
                    "Warning: Failed to create mistralrs-server symlink in {} directory: {}",
                    build_profile, e
                );
            }
        }
    }

    // Also run the default Tauri build script
    tauri_build::build();
}
