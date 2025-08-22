use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn download_binary(
    url: &str,
    target_path: &Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn setup_pdfium(
    target: &str,
    target_dir: &Path,
    out_dir: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
        let pdfium_temp_dir = Path::new(out_dir).join("pdfium-download");
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
                            let mut perms =
                                fs::metadata(&pdfium_target_path).unwrap().permissions();
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
            eprintln!(
                "Warning: Failed to copy PDFium to {:?} directory: {}",
                profile_dir, e
            );
        } else {
            println!("Successfully copied PDFium to {:?}", profile_dir);
        }

        // Make bundle version executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if target_path_standardized.exists() {
                let mut bundle_perms = fs::metadata(&target_path_standardized)
                    .unwrap()
                    .permissions();
                bundle_perms.set_mode(0o755);
                fs::set_permissions(&target_path_standardized, bundle_perms).unwrap();
            }
        }
    }

    Ok(pdfium_target_path)
}

pub fn setup_pdfium_env(target: &str, pdfium_target_path: &Path, pdfium_dir: &Path) {
    // Set environment variables for PDFium dynamic library path
    if pdfium_target_path.exists() {
        let pdfium_dir_str = pdfium_dir.to_string_lossy();

        // Set PDFIUM_DYNAMIC_LIB_PATH for pdfium-render crate
        println!("cargo:rustc-env=PDFIUM_DYNAMIC_LIB_PATH={}", pdfium_dir_str);

        // For runtime, set the library path environment variable
        if target.contains("windows") {
            println!(
                "cargo:rustc-env=PATH={};{}",
                pdfium_dir_str,
                env::var("PATH").unwrap_or_default()
            );
        } else if target.contains("darwin") {
            println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", pdfium_dir_str);
        } else {
            println!("cargo:rustc-env=LD_LIBRARY_PATH={}", pdfium_dir_str);
        }

        println!("cargo:rustc-env=PDFIUM_LIB_PATH={}", pdfium_dir_str);
    }
}
