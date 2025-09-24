use std::fs;
use std::path::{Path, PathBuf};

const UV_VERSION: &str = "0.8.22";

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

fn extract_uv_binary(
    archive_path: &Path,
    target_dir: &Path,
    binary_name: &str,
    is_tar_gz: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;

    if is_tar_gz {
        // Extract TAR.GZ file
        let tar_gz = fs::File::open(archive_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let filename = path.to_string_lossy();

            // Look for uv binary (may be in bin/ directory or root)
            if filename.ends_with("uv") || filename.ends_with("/uv") {
                let output_path = target_dir.join(binary_name);
                entry.unpack(&output_path)?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&output_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&output_path, perms)?;
                }

                return Ok(output_path);
            }
        }
    } else {
        // Extract ZIP file
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let filename = file.name();

            // Look for uv or uv.exe (may be in bin/ directory or root)
            if filename.ends_with("uv") || filename.ends_with("uv.exe") || filename.ends_with("bin/uv") {
                let output_path = target_dir.join(binary_name);

                let mut outfile = fs::File::create(&output_path)?;
                std::io::copy(&mut file, &mut outfile)?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&output_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&output_path, perms)?;
                }

                return Ok(output_path);
            }
        }
    }

    Err("UV binary not found in archive".into())
}

fn get_uv_asset_info(target: &str) -> Result<(&'static str, &'static str), Box<dyn std::error::Error>> {
    let (asset, binary) = if target.contains("windows") {
        if target.contains("aarch64") {
            ("uv-aarch64-pc-windows-msvc.zip", "uv.exe")
        } else {
            ("uv-x86_64-pc-windows-msvc.zip", "uv.exe")
        }
    } else if target.contains("darwin") {
        if target.contains("aarch64") {
            ("uv-aarch64-apple-darwin.tar.gz", "uv")
        } else {
            ("uv-x86_64-apple-darwin.tar.gz", "uv")
        }
    } else if target.contains("linux") {
        if target.contains("aarch64") {
            ("uv-aarch64-unknown-linux-gnu.tar.gz", "uv")
        } else {
            ("uv-x86_64-unknown-linux-gnu.tar.gz", "uv")
        }
    } else {
        return Err(format!("Unsupported target: {}", target).into());
    };

    Ok((asset, binary))
}

pub fn setup_uv(
    target: &str,
    target_dir: &Path,
    out_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use dedicated UV directory
    let uv_dir = target_dir.join("uv");
    fs::create_dir_all(&uv_dir)?;

    let (asset_name, binary_name) = get_uv_asset_info(target)?;
    let uv_target_path = uv_dir.join(binary_name);

    if !uv_target_path.exists() {
        let url = format!(
            "https://github.com/astral-sh/uv/releases/download/{}/{}",
            UV_VERSION, asset_name
        );

        let temp_dir = Path::new(out_dir).join("uv_temp");
        fs::create_dir_all(&temp_dir)?;
        let archive_path = temp_dir.join(asset_name);

        download_binary(&url, &archive_path, "UV")?;
        extract_uv_binary(&archive_path, &uv_dir, binary_name, asset_name.ends_with(".tar.gz"))?;

        // Clean up temporary files
        fs::remove_dir_all(&temp_dir).ok();
    } else {
        println!("UV binary already exists at {:?}", uv_target_path);
    }

    println!("UV binary available at: {}", uv_target_path.display());

    Ok(())
}