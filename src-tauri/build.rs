use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const GIT_LFS_VERSION: &str = "v3.7.0";

fn download_git_lfs(url: &str, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading git-lfs from: {}", url);
    
    let response = ureq::get(url).call()?;
    let mut reader = response.into_reader();
    
    let mut file = fs::File::create(target_path)?;
    std::io::copy(&mut reader, &mut file)?;
    
    Ok(())
}

fn extract_git_lfs(archive_path: &Path, target_dir: &Path, is_zip: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(target_dir)?;
    
    if is_zip {
        // Extract ZIP file
        let file = fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let filename = file.name();
            
            if filename.ends_with("git-lfs") || filename.ends_with("git-lfs.exe") {
                let binary_name = if filename.ends_with(".exe") { "git-lfs.exe" } else { "git-lfs" };
                let output_path = target_dir.join(binary_name);
                
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
                let output_path = target_dir.join("git-lfs");
                entry.unpack(&output_path)?;
                return Ok(output_path);
            }
        }
    }
    
    Err("git-lfs binary not found in archive".into())
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
    
    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();
    
    // Determine the target binary name
    let target_binary_name = if target.contains("windows") { "git-lfs.exe" } else { "git-lfs" };
    let target_path = target_dir.join(target_binary_name);
    
    // Skip download if binary already exists
    if target_path.exists() {
        println!("git-lfs binary already exists at {:?}", target_path);
    } else {
        // Create a temporary directory for download
        let temp_dir = Path::new(&out_dir).join("git-lfs-download");
        fs::create_dir_all(&temp_dir).unwrap();
        
        // Construct the download URL and filename
        let archive_name = format!("git-lfs-{}-{}-{}.{}", platform, arch, GIT_LFS_VERSION, extension);
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
        let extracted_path = extract_git_lfs(&archive_path, &temp_dir, extension == "zip")
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
    
    // Also run the default Tauri build script
    tauri_build::build();
}
