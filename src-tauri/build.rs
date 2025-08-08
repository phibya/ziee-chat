use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PANDOC_VERSION: &str = "3.7.0.2";

fn download_binary(url: &str, target_path: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading {} from: {}", name, url);

    let response = ureq::get(url).call()?;
    let mut reader = response.into_reader();

    let mut file = fs::File::create(target_path)?;
    std::io::copy(&mut reader, &mut file)?;

    Ok(())
}

fn download_pdfium(url: &str, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    download_binary(url, target_path, "PDFium")
}

fn download_pandoc(url: &str, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    download_binary(url, target_path, "Pandoc")
}

fn extract_pdfium(
    archive_path: &Path,
    target_dir: &Path,
    target_binary_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;

    // Extract tar.gz file
    let tar_gz = fs::File::open(archive_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    // PDFium dynamic libraries are typically in lib/ directory
    let library_names = if target_binary_name.contains("windows") {
        vec!["bin/pdfium.dll", "lib/pdfium.dll"]
    } else if target_binary_name.contains("darwin") {
        vec!["lib/libpdfium.dylib"]
    } else {
        vec!["lib/libpdfium.so"]
    };

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_string_lossy();

        // Check if this is the PDFium library we're looking for
        if library_names.iter().any(|name| path_str.ends_with(name)) {
            let output_path = target_dir.join(target_binary_name);
            entry.unpack(&output_path)?;
            return Ok(output_path);
        }
    }

    Err("PDFium library not found in archive".into())
}

fn extract_pandoc(
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

            // Look for pandoc or pandoc.exe (may be in bin/ directory or root)
            if filename.ends_with("pandoc") || filename.ends_with("pandoc.exe") || filename.ends_with("bin/pandoc") {
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
            let path_str = path.to_string_lossy();

            // Look for pandoc (may be in bin/ directory or root)
            if path_str.ends_with("pandoc") || path_str.ends_with("pandoc.exe") || path_str.ends_with("bin/pandoc") {
                let output_path = target_dir.join(target_binary_name);
                entry.unpack(&output_path)?;
                return Ok(output_path);
            }
        }
    }

    Err("Pandoc binary not found in archive".into())
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
    
    // Set PostgreSQL version for postgresql_embedded crate
    env::set_var("POSTGRESQL_VERSION", "17.5.0");
    println!("cargo:rustc-env=POSTGRESQL_VERSION=17.5.0");
    println!("Setting PostgreSQL version to 17.5.0");

    // Get the output directory and build profile
    let out_dir = env::var("OUT_DIR").unwrap();
    let _profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

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

    // === PDFium Binary Download ===
    
    // Use dedicated PDFium directory
    let pdfium_dir = target_dir.join("pdfium");
    fs::create_dir_all(&pdfium_dir)
        .unwrap_or_else(|e| panic!("Failed to create PDFium directory: {}", e));

    // Map target to PDFium platform names
    let (pdfium_platform, pdfium_arch) = if target.contains("windows") {
        if target.contains("x86_64") {
            ("win", "x64")
        } else if target.contains("aarch64") {
            ("win", "arm64")
        } else {
            panic!("Unsupported Windows architecture for PDFium: {}", target);
        }
    } else if target.contains("darwin") {
        if target.contains("x86_64") {
            ("mac", "x64")
        } else if target.contains("aarch64") {
            ("mac", "arm64")
        } else {
            panic!("Unsupported macOS architecture for PDFium: {}", target);
        }
    } else if target.contains("linux") {
        if target.contains("x86_64") {
            ("linux", "x64")
        } else if target.contains("aarch64") {
            ("linux", "arm64")
        } else {
            panic!("Unsupported Linux architecture for PDFium: {}", target);
        }
    } else {
        panic!("Unsupported platform for PDFium: {}", target);
    };

    // Use target triple format for dynamic library naming
    let pdfium_binary_name = if target.contains("windows") {
        format!("pdfium-{}.dll", target)
    } else if target.contains("darwin") {
        format!("libpdfium-{}.dylib", target)
    } else {
        format!("libpdfium-{}.so", target)
    };

    let pdfium_target_path = pdfium_dir.join(&pdfium_binary_name);
    
    println!("PDFium target path:  {:?}", pdfium_target_path);

    // Download PDFium if it doesn't exist
    if !pdfium_target_path.exists() {
        println!("Downloading PDFium library...");
        
        // Create a temporary directory for PDFium download
        let pdfium_temp_dir = Path::new(&out_dir).join("pdfium-download");
        fs::create_dir_all(&pdfium_temp_dir).unwrap();

        // Construct the PDFium download URL for dynamic libraries
        // Format: https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-platform-arch.tgz
        let pdfium_archive_name = format!("pdfium-{}-{}.tgz", pdfium_platform, pdfium_arch);
        let pdfium_download_url = format!(
            "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/{}",
            pdfium_archive_name
        );

        let pdfium_archive_path = pdfium_temp_dir.join(&pdfium_archive_name);

        // Download the PDFium archive
        if let Err(e) = download_pdfium(&pdfium_download_url, &pdfium_archive_path) {
            eprintln!("Warning: Failed to download PDFium: {}", e);
            eprintln!("PDF thumbnail generation will not be available");
        } else {
            // Extract the PDFium library
            match extract_pdfium(&pdfium_archive_path, &pdfium_temp_dir, &pdfium_binary_name) {
                Ok(extracted_path) => {
                    // Copy to target directory with platform-specific name
                    if let Err(e) = fs::copy(&extracted_path, &pdfium_target_path) {
                        eprintln!("Warning: Failed to copy PDFium binary: {}", e);
                    } else {
                        println!("Successfully installed PDFium to {:?}", pdfium_target_path);
                        
                        // Make it executable on Unix
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = fs::metadata(&pdfium_target_path).unwrap().permissions();
                            perms.set_mode(0o755);
                            fs::set_permissions(&pdfium_target_path, perms).unwrap();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to extract PDFium: {}", e);
                }
            }
        }

        // Clean up temporary files
        fs::remove_dir_all(&pdfium_temp_dir).ok();
    } else {
        println!("PDFium binary already exists at {:?}", pdfium_target_path);
    }
    
    // Always copy to lib directories for Tauri bundling (during every build)
    if pdfium_target_path.exists() {
        let standardized_name = if target.contains("windows") {
            "pdfium.dll"
        } else if target.contains("darwin") {
            "libpdfium.dylib"
        } else {
            "libpdfium.so"
        };

        let profile_dir = pdfium_dir.join("target");
        fs::create_dir_all(&profile_dir).ok();

        let target_path_standardized = profile_dir.join(standardized_name);

        if let Err(e) = fs::copy(&pdfium_target_path, &target_path_standardized) {
            eprintln!("Warning: Failed to copy PDFium to {:?} directory: {}", profile_dir, e);
        } else {
            println!("Successfully copied PDFium to {:?}", profile_dir);
        }

        // Make bundle version executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if target_path_standardized.exists() {
                let mut bundle_perms = fs::metadata(&target_path_standardized).unwrap().permissions();
                bundle_perms.set_mode(0o755);
                fs::set_permissions(&target_path_standardized, bundle_perms).unwrap();
            }
        }
    }

    // === Pandoc Binary Download ===
    
    // Use dedicated Pandoc directory
    let pandoc_dir = target_dir.join("pandoc");
    fs::create_dir_all(&pandoc_dir)
        .unwrap_or_else(|e| panic!("Failed to create Pandoc directory: {}", e));

    // Map target to Pandoc platform names (based on actual GitHub release assets)
    let (pandoc_platform, pandoc_arch, pandoc_extension) = if target.contains("windows") {
        if target.contains("x86_64") {
            ("windows", "x86_64", "zip")
        } else {
            panic!("Unsupported Windows architecture for Pandoc: {}", target);
        }
    } else if target.contains("darwin") {
        if target.contains("x86_64") {
            ("x86_64", "macOS", "zip")
        } else if target.contains("aarch64") {
            ("arm64", "macOS", "zip")
        } else {
            panic!("Unsupported macOS architecture for Pandoc: {}", target);
        }
    } else if target.contains("linux") {
        if target.contains("x86_64") {
            ("linux", "amd64", "tar.gz")
        } else if target.contains("aarch64") {
            ("linux", "arm64", "tar.gz")
        } else {
            panic!("Unsupported Linux architecture for Pandoc: {}", target);
        }
    } else {
        panic!("Unsupported platform for Pandoc: {}", target);
    };

    // Use target triple format for binary naming
    let pandoc_binary_name = if target.contains("windows") {
        format!("pandoc-{}.exe", target)
    } else {
        format!("pandoc-{}", target)
    };

    let pandoc_target_path = pandoc_dir.join(&pandoc_binary_name);
    
    println!("Pandoc target path: {:?}", pandoc_target_path);

    // Download Pandoc if it doesn't exist
    if !pandoc_target_path.exists() {
        println!("Downloading Pandoc binary...");
        
        // Create a temporary directory for Pandoc download
        let pandoc_temp_dir = Path::new(&out_dir).join("pandoc-download");
        fs::create_dir_all(&pandoc_temp_dir).unwrap();

        // Construct the Pandoc download URL based on actual GitHub release assets
        // Format varies: Windows: pandoc-{version}-windows-{arch}.zip, macOS: pandoc-{version}-{arch}-macOS.zip, Linux: pandoc-{version}-linux-{arch}.tar.gz
        let pandoc_archive_name = if target.contains("windows") {
            format!("pandoc-{}-{}-{}.{}", PANDOC_VERSION, pandoc_platform, pandoc_arch, pandoc_extension)
        } else if target.contains("darwin") {
            format!("pandoc-{}-{}-{}.{}", PANDOC_VERSION, pandoc_platform, pandoc_arch, pandoc_extension)
        } else {
            format!("pandoc-{}-{}-{}.{}", PANDOC_VERSION, pandoc_platform, pandoc_arch, pandoc_extension)
        };
        let pandoc_download_url = format!(
            "https://github.com/jgm/pandoc/releases/download/{}/{}",
            PANDOC_VERSION, pandoc_archive_name
        );

        let pandoc_archive_path = pandoc_temp_dir.join(&pandoc_archive_name);

        // Download the Pandoc archive
        if let Err(e) = download_pandoc(&pandoc_download_url, &pandoc_archive_path) {
            eprintln!("Warning: Failed to download Pandoc: {}", e);
            eprintln!("Pandoc functionality will not be available");
        } else {
            // Extract the Pandoc binary
            match extract_pandoc(&pandoc_archive_path, &pandoc_temp_dir, pandoc_extension == "zip", &pandoc_binary_name) {
                Ok(extracted_path) => {
                    // Copy to target directory with platform-specific name
                    if let Err(e) = fs::copy(&extracted_path, &pandoc_target_path) {
                        eprintln!("Warning: Failed to copy Pandoc binary: {}", e);
                    } else {
                        println!("Successfully installed Pandoc to {:?}", pandoc_target_path);
                        
                        // Make it executable on Unix
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = fs::metadata(&pandoc_target_path).unwrap().permissions();
                            perms.set_mode(0o755);
                            fs::set_permissions(&pandoc_target_path, perms).unwrap();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to extract Pandoc: {}", e);
                }
            }
        }

        // Clean up temporary files
        fs::remove_dir_all(&pandoc_temp_dir).ok();
    } else {
        println!("Pandoc binary already exists at {:?}", pandoc_target_path);
    }
    
    // Always copy to bin directories for Tauri bundling (during every build)
    if pandoc_target_path.exists() {
        let standardized_name = if target.contains("windows") {
            "pandoc.exe"
        } else {
            "pandoc"
        };
        
        // Copy to both debug and release bin directories
        for build_profile in ["debug", "release"] {
            let profile_dir = target_dir.join(build_profile);
            let bin_dir = profile_dir.join("bin");
            fs::create_dir_all(&bin_dir).ok();
            
            let pandoc_bin_path = bin_dir.join(standardized_name);
            if let Err(e) = fs::copy(&pandoc_target_path, &pandoc_bin_path) {
                eprintln!("Warning: Failed to copy Pandoc to {} bin directory: {}", build_profile, e);
            } else {
                println!("Successfully copied Pandoc to {} bin directory: {:?}", build_profile, pandoc_bin_path);
            }
            
            // Make bin version executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if pandoc_bin_path.exists() {
                    let mut bin_perms = fs::metadata(&pandoc_bin_path).unwrap().permissions();
                    bin_perms.set_mode(0o755);
                    fs::set_permissions(&pandoc_bin_path, bin_perms).unwrap();
                }
            }
        }
    }


    // Build mistralrs-server
    println!("cargo:rerun-if-changed=mistralrs-server");
    let _mistralrs_path = match build_mistralrs_server(&target_dir, &target) {
        Ok(path) => Some(path),
        Err(e) => {
            eprintln!("Warning: Failed to build mistralrs-server: {}", e);
            eprintln!("Continuing without mistralrs-server binary");
            None
        }
    };
    
    // Set environment variables for PDFium dynamic library path
    if pdfium_target_path.exists() {
        let pdfium_dir_str = pdfium_dir.to_string_lossy();
        
        // Set PDFIUM_DYNAMIC_LIB_PATH for pdfium-render crate
        println!("cargo:rustc-env=PDFIUM_DYNAMIC_LIB_PATH={}", pdfium_dir_str);
        
        // For runtime, set the library path environment variable
        if target.contains("windows") {
            println!("cargo:rustc-env=PATH={};{}", pdfium_dir_str, env::var("PATH").unwrap_or_default());
        } else if target.contains("darwin") {
            println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", pdfium_dir_str);
        } else {
            println!("cargo:rustc-env=LD_LIBRARY_PATH={}", pdfium_dir_str);
        }
        
        println!("cargo:rustc-env=PDFIUM_LIB_PATH={}", pdfium_dir_str);
    }

    // Also run the default Tauri build script
    tauri_build::build();
}
