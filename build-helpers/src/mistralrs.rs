use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn build_mistralrs_server(
    target_dir: &Path,
    target: &str,
    source_path: Option<&Path>
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    println!("Building mistralrs-server...");

    // Default source path if not provided  
    let default_path;
    let mistralrs_dir = if let Some(path) = source_path {
        path
    } else {
        default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("../src-engines/mistralrs-server");
        &default_path
    };

    if !mistralrs_dir.exists() {
        return Err(format!("mistralrs-server source directory not found at: {}", mistralrs_dir.display()).into());
    }

    // Use simple binary naming (no platform suffix)
    let binary_name = if target.contains("windows") {
        "mistralrs-server.exe"
    } else {
        "mistralrs-server"
    };

    // Use dedicated mistralrs-build directory
    let mistralrs_build_dir = target_dir.join("mistralrs-build");
    fs::create_dir_all(&mistralrs_build_dir)?;
    
    // Create bin directory for the final binary
    let bin_dir = mistralrs_build_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    let target_path = bin_dir.join(binary_name);

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

    // Copy to mistralrs-build/bin directory with simple name
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