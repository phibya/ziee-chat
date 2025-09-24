use std::fs;
use std::path::{Path, PathBuf};

const BUN_VERSION: &str = "1.2.22";

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

fn extract_bun_binary(
    archive_path: &Path,
    target_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;

    // Extract ZIP file
    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let filename = file.name();

        // Look for bun or bun.exe (may be in bin/ directory or root)
        if filename.ends_with("bun") || filename.ends_with("bun.exe") || filename.ends_with("bin/bun") {
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

    Err("Bun binary not found in archive".into())
}

fn get_bun_asset_info(target: &str) -> Result<(&'static str, &'static str), Box<dyn std::error::Error>> {
    let (asset, binary) = if target.contains("windows") {
        ("bun-windows-x64.zip", "bun.exe")
    } else if target.contains("darwin") {
        if target.contains("aarch64") {
            ("bun-darwin-aarch64.zip", "bun")
        } else {
            ("bun-darwin-x64.zip", "bun")
        }
    } else if target.contains("linux") {
        if target.contains("aarch64") {
            ("bun-linux-aarch64.zip", "bun")
        } else {
            ("bun-linux-x64.zip", "bun")
        }
    } else {
        return Err(format!("Unsupported target: {}", target).into());
    };

    Ok((asset, binary))
}

pub fn setup_bun(
    target: &str,
    target_dir: &Path,
    out_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use dedicated Bun directory
    let bun_dir = target_dir.join("bun");
    fs::create_dir_all(&bun_dir)?;

    let (asset_name, binary_name) = get_bun_asset_info(target)?;
    let bun_target_path = bun_dir.join(binary_name);

    if !bun_target_path.exists() {
        let url = format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}",
            BUN_VERSION, asset_name
        );

        let temp_dir = Path::new(out_dir).join("bun_temp");
        fs::create_dir_all(&temp_dir)?;
        let archive_path = temp_dir.join(asset_name);

        download_binary(&url, &archive_path, "Bun")?;
        extract_bun_binary(&archive_path, &bun_dir, binary_name)?;

        // Clean up temporary files
        fs::remove_dir_all(&temp_dir).ok();
    } else {
        println!("Bun binary already exists at {:?}", bun_target_path);
    }

    println!("Bun binary available at: {}", bun_target_path.display());

    Ok(())
}