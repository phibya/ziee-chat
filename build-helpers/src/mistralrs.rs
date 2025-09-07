use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Enable GPU only when explicitly requested
fn wants_gpu() -> bool {
    matches!(env::var("ZIEE_USE_GPU").as_deref(), Ok("1") | Ok("true") | Ok("on"))
}

/// Get comprehensive feature set for the target platform (GPU only if requested)
fn nvcc_available() -> bool {
    Command::new(if cfg!(windows) { "nvcc.exe" } else { "nvcc" })
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_comprehensive_features(target: &str) -> String {
    // Opt-in GPU: set ZIEE_USE_GPU=1|true|on
    if !wants_gpu() {
        return String::new(); // CPU-only default
    }

    if target.contains("darwin") && (target.contains("aarch64") || target.contains("arm64")) {
        "metal,accelerate".to_string()
    } else if target.contains("darwin") && target.contains("x86_64") {
        "accelerate,mkl".to_string()
    } else if target.contains("linux") {
        if nvcc_available() { "cuda,vulkan,mkl,flash-attn,cudnn".to_string() } else { String::new() }
    } else if target.contains("windows") {
        if nvcc_available() { "cuda,mkl,flash-attn,cudnn".to_string() } else { String::new() }
    } else {
        String::new()
    }
}

/// Build mistralrs-server with comprehensive features for the platform
pub fn build(
    target_dir: &Path,
    target: &str,
    source_path: Option<&Path>,
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
        return Err(format!(
            "mistralrs-server source directory not found at: {}",
            mistralrs_dir.display()
        )
            .into());
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
        .arg("--release")   // Always build in release mode
        .arg("--no-default-features"); // CPU baseline on all OSes

    // Add GPU features only when requested
    let features = get_comprehensive_features(target);
    if !features.is_empty() {
        cmd.arg("--features").arg(features);
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
