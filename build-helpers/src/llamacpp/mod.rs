pub mod backend;
pub mod config;
pub mod platform;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::llamacpp::backend::{get_multi_backend_config, BackendConfig, BackendType};
use crate::llamacpp::config::BuildConfig;
use crate::llamacpp::platform::get_platform_config;

/// Get comprehensive backend configuration for the target platform
fn get_comprehensive_backend_config(
    target: &str,
) -> Result<BackendConfig, Box<dyn std::error::Error>> {
    if target.contains("darwin") || target.contains("apple") {
        // macOS: Use comprehensive single backend config with Metal/Accelerate
        get_comprehensive_macos_config(target)
    } else {
        // Linux/Windows: Use multi-backend config with all GPU support
        get_multi_backend_config(target)
    }
}

/// Get comprehensive macOS configuration with all available features
fn get_comprehensive_macos_config(
    target: &str,
) -> Result<BackendConfig, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    let mut cmake_flags = HashMap::new();
    let dependencies = vec![];

    // Enable CPU with all optimizations
    cmake_flags.insert("GGML_CPU".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());

    // Enable backend dynamic loading
    cmake_flags.insert("GGML_BACKEND_DL".to_string(), "ON".to_string());

    if target.contains("aarch64") || target.contains("arm64") {
        // Apple Silicon: Metal + Accelerate
        cmake_flags.insert("GGML_METAL".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_USE_BF16".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_EMBED_LIBRARY".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_SHADER_DEBUG".to_string(), "OFF".to_string());
        cmake_flags.insert("GGML_ACCELERATE".to_string(), "ON".to_string());
        cmake_flags.insert("CMAKE_OSX_ARCHITECTURES".to_string(), "arm64".to_string());
    } else {
        // Intel Mac: Accelerate + SIMD optimizations
        cmake_flags.insert("GGML_ACCELERATE".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_AVX2".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_AVX".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_SSE3".to_string(), "ON".to_string());
        cmake_flags.insert("CMAKE_OSX_ARCHITECTURES".to_string(), "x86_64".to_string());
        cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());
    }

    // Disable other GPU backends on macOS
    cmake_flags.insert("GGML_CUDA".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_VULKAN".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_OPENCL".to_string(), "OFF".to_string());

    Ok(BackendConfig {
        name: "macOS Comprehensive".to_string(),
        backend_type: if target.contains("aarch64") || target.contains("arm64") {
            BackendType::Metal
        } else {
            BackendType::BLAS
        },
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies,
    })
}

/// Build llama.cpp with comprehensive features for the platform
pub fn build(
    target_dir: &Path,
    target: &str,
    source_path: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let platform_config = get_platform_config(target)?;

    // Use comprehensive backend configuration that includes all platform features
    let comprehensive_config = get_comprehensive_backend_config(target)?;

    // Default source path if not provided
    let default_path;
    let source_dir = if let Some(path) = source_path {
        path
    } else {
        default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("../src-engines/llama.cpp");
        &default_path
    };

    if !source_dir.exists() {
        return Err(format!(
            "llama.cpp source directory not found at: {}",
            source_dir.display()
        )
        .into());
    }

    let build_config = BuildConfig {
        source_dir: source_dir.to_path_buf(),
        target_dir: target_dir.to_path_buf(),
        target: target.to_string(),
        platform: platform_config,
        backends: vec![comprehensive_config],
    };

    build_with_config(&build_config)
}

/// Build llama.cpp with the given configuration
fn build_with_config(config: &BuildConfig) -> Result<PathBuf, Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", config.source_dir.display());
    let backend_names: Vec<String> = config.backends.iter().map(|b| b.name.clone()).collect();
    println!(
        "Building llama.cpp with backends [{}] for {}",
        backend_names.join(", "),
        config.target
    );

    let build_dir = config.target_dir.join("llamacpp-build");
    fs::create_dir_all(&build_dir)?;

    // Generate CMAKE command
    let mut cmake_cmd = Command::new("cmake");
    cmake_cmd.current_dir(&config.source_dir);
    cmake_cmd.args(["-B", &build_dir.to_string_lossy()]);
    cmake_cmd.args(["-S", "."]);
    cmake_cmd.arg("-DCMAKE_BUILD_TYPE=Release");

    // Set up proper library paths for the final structure (bin/ and lib/)
    let install_prefix = config.target_dir.join("llamacpp");
    cmake_cmd.arg(format!(
        "-DCMAKE_INSTALL_PREFIX={}",
        install_prefix.to_string_lossy()
    ));

    // Configure runtime library paths based on platform
    if config.target.contains("darwin") || config.target.contains("apple") {
        // macOS: Set RPATH to look in the same directory
        cmake_cmd.arg("-DCMAKE_BUILD_RPATH=@loader_path");
        cmake_cmd.arg("-DCMAKE_INSTALL_RPATH=@loader_path");
        cmake_cmd.arg("-DCMAKE_BUILD_WITH_INSTALL_RPATH=ON");
        cmake_cmd.arg("-DCMAKE_INSTALL_RPATH_USE_LINK_PATH=OFF"); // Disable to prevent path corruption
    } else if config.target.contains("linux") {
        // Linux: Set RPATH to look in the same directory
        cmake_cmd.arg("-DCMAKE_BUILD_RPATH=$ORIGIN");
        cmake_cmd.arg("-DCMAKE_INSTALL_RPATH=$ORIGIN");
        cmake_cmd.arg("-DCMAKE_BUILD_WITH_INSTALL_RPATH=ON");
        cmake_cmd.arg("-DCMAKE_INSTALL_RPATH_USE_LINK_PATH=OFF");
    } else if config.target.contains("windows") {
        // Windows: DLL search path includes executable directory
        // This is handled by placing DLLs in the same directory as the executable
    }

    // Add platform-specific flags
    for (key, value) in &config.platform.cmake_flags {
        cmake_cmd.arg(format!("-D{}={}", key, value));
    }

    // Combine all backend flags - later backends override earlier ones
    let mut combined_cmake_flags = std::collections::HashMap::new();
    for backend in &config.backends {
        for (key, value) in &backend.cmake_flags {
            combined_cmake_flags.insert(key.clone(), value.clone());
        }
    }

    // Add combined backend-specific flags
    for (key, value) in &combined_cmake_flags {
        cmake_cmd.arg(format!("-D{}={}", key, value));
    }

    // Common llama.cpp build settings - only build server
    cmake_cmd.arg("-DLLAMA_BUILD_TESTS=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_EXAMPLES=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_SERVER=ON");
    // Disable building of other binaries
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_CLI=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_RUN=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_BENCH=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_QUANTIZE=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_PERPLEXITY=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_BATCHED_BENCH=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_TTS=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_GGUF_SPLIT=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_IMATRIX=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_TOKENIZE=OFF");
    cmake_cmd.arg("-DLLAMA_BUILD_LLAMA_MTMD_CLI=OFF");
    // Enable backend features
    cmake_cmd.arg("-DGGML_BACKEND_DL=ON");
    cmake_cmd.arg("-DGGML_CPU_ALL_VARIANTS=ON");


    // Install configuration
    cmake_cmd.arg("-DCMAKE_INSTALL_BINDIR=bin");
    cmake_cmd.arg("-DCMAKE_INSTALL_LIBDIR=lib");
    cmake_cmd.arg("-DCMAKE_INSTALL_INCLUDEDIR=include");

    // Set environment variables if needed
    for (key, value) in &config.platform.env_vars {
        cmake_cmd.env(key, value);
    }

    // Combine all backend environment variables
    let mut combined_env_vars = std::collections::HashMap::new();
    for backend in &config.backends {
        for (key, value) in &backend.env_vars {
            combined_env_vars.insert(key.clone(), value.clone());
        }
    }

    for (key, value) in &combined_env_vars {
        cmake_cmd.env(key, value);
    }

    println!("Running CMake configure: {:?}", cmake_cmd);
    let output = cmake_cmd.output()?;

    if !output.status.success() {
        return Err(format!(
            "CMake configure failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // Build the project
    let mut build_cmd = Command::new("cmake");
    build_cmd.args(["--build", &build_dir.to_string_lossy()]);
    build_cmd.args(["--config", "Release"]);

    // Use parallel build if available
    if let Ok(cores) = std::thread::available_parallelism() {
        build_cmd.args(["-j", &cores.get().to_string()]);
    }

    println!("Running CMake build: {:?}", build_cmd);
    let output = build_cmd.output()?;

    if !output.status.success() {
        return Err(format!(
            "CMake build failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // Install the project to set up proper directory structure
    let mut install_cmd = Command::new("cmake");
    install_cmd.args(["--install", &build_dir.to_string_lossy()]);
    install_cmd.args(["--config", "Release"]);

    println!("Running CMake install: {:?}", install_cmd);
    let output = install_cmd.output()?;

    if !output.status.success() {
        return Err(format!(
            "CMake install failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // The install step should have created the proper structure
    let llamacpp_target_dir = config.target_dir.join("llamacpp");
    let bin_dir_target = llamacpp_target_dir.join("bin");

    // Ensure bin directory exists
    fs::create_dir_all(&bin_dir_target)?;

    // Find the installed binary
    let target_binary = bin_dir_target.join(if config.target.contains("windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    });

    // If cmake install didn't work as expected, fall back to manual copy
    if !target_binary.exists() {
        let bin_dir = build_dir.join("bin");
        let server_binary = find_binary(&bin_dir, &["llama-server", "llama-server.exe"])?;
        fs::copy(&server_binary, &target_binary)?;

        // Copy all required runtime libraries and resources to bin directory
        copy_runtime_files(&bin_dir, &bin_dir_target, &config.target)?;

        println!(
            "Manually copied llama.cpp server: {}",
            target_binary.display()
        );
        println!("Runtime libraries copied to: {}", bin_dir_target.display());
    } else {
        // Copy all libraries to bin directory
        ensure_runtime_libraries(&build_dir, &bin_dir_target, &config.target)?;

        println!("Installed llama.cpp server: {}", target_binary.display());
        println!("All files in: {}", bin_dir_target.display());
    }

    // Clean up the installation - remove unnecessary files
    cleanup_installation(&llamacpp_target_dir)?;

    // Verify the binary can find its libraries
    verify_binary_dependencies(&target_binary, &bin_dir_target)?;

    Ok(target_binary)
}

/// Copy runtime files needed for llama-server to run
fn copy_runtime_files(
    bin_dir: &Path,
    target_dir: &Path,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Copy all shared libraries (.dylib, .so, .dll)
    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        // Copy shared libraries
        if file_name.ends_with(".dylib")
            || file_name.ends_with(".so")
            || file_name.ends_with(".dll")
        {
            let target_path = target_dir.join(&*file_name);
            fs::copy(&path, &target_path)?;
            println!("Copied runtime library: {}", file_name);
        }

        // Copy Metal resources (macOS specific)
        if (target.contains("darwin") || target.contains("apple"))
            && (file_name.ends_with(".metallib")
                || file_name.ends_with(".metal")
                || file_name.ends_with(".h"))
        {
            let target_path = target_dir.join(&*file_name);
            fs::copy(&path, &target_path)?;
            println!("Copied Metal resource: {}", file_name);
        }
    }

    Ok(())
}

/// Clean up the installation - remove unnecessary files
fn cleanup_installation(llamacpp_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = llamacpp_dir.join("bin");

    // Move any files from root to bin directory
    for entry in fs::read_dir(llamacpp_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();

            // Move all libraries and resources to bin directory
            if file_name.ends_with(".dylib")
                || file_name.ends_with(".so")
                || file_name.ends_with(".dll")
                || file_name.ends_with(".metallib")
                || file_name.ends_with(".metal")
                || file_name.ends_with(".h")
            {
                let target_path = bin_dir.join(&*file_name);
                if !target_path.exists() {
                    fs::rename(&path, &target_path)?;
                    println!("Moved {} to bin/", file_name);
                } else {
                    fs::remove_file(&path)?;
                    println!("Removed duplicate: {}", file_name);
                }
            }
            // Remove other executables (we only want llama-server)
            else if file_name.starts_with("llama-")
                && file_name != "llama-server"
                && file_name != "llama-server.exe"
            {
                fs::remove_file(&path)?;
                println!("Removed unnecessary executable: {}", file_name);
            }
        }
    }

    // Clean up bin directory - keep only llama-server and required libraries
    for entry in fs::read_dir(&bin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();

            // Keep llama-server executable
            if file_name == "llama-server" || file_name == "llama-server.exe" {
                continue;
            }

            // Keep all libraries and resources
            if file_name.ends_with(".dylib")
                || file_name.ends_with(".so")
                || file_name.ends_with(".dll")
                || file_name.ends_with(".metallib")
                || file_name.ends_with(".metal")
                || file_name.ends_with(".h")
            {
                println!("Keeping in bin/: {}", file_name);
                continue;
            }

            // Remove other executables
            fs::remove_file(&path)?;
            println!(
                "Removed non-llama-server executable from bin/: {}",
                file_name
            );
        }
    }

    // Remove lib directory if it exists and move everything to bin
    let lib_dir = llamacpp_dir.join("lib");
    if lib_dir.exists() {
        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let target_path = bin_dir.join(&*file_name);

                if !target_path.exists() {
                    fs::rename(&path, &target_path)?;
                    println!("Moved from lib/ to bin/: {}", file_name);
                }
            }
        }

        // Remove empty lib directory
        let _ = fs::remove_dir_all(&lib_dir);
    }

    Ok(())
}

/// Ensure runtime libraries are available in the bin directory
fn ensure_runtime_libraries(
    build_dir: &Path,
    bin_dir: &Path,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_bin_dir = build_dir.join("bin");
    if source_bin_dir.exists() {
        copy_runtime_files(&source_bin_dir, bin_dir, target)?;
    }

    // Also check for libraries in build/lib
    let source_lib_dir = build_dir.join("lib");
    if source_lib_dir.exists() {
        copy_runtime_files(&source_lib_dir, bin_dir, target)?;
    }

    Ok(())
}

/// Verify that the binary can find its runtime dependencies
fn verify_binary_dependencies(
    binary_path: &Path,
    bin_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !binary_path.exists() {
        return Err(format!("Binary not found: {}", binary_path.display()).into());
    }

    // Make the binary executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(binary_path, perms)?;
    }

    println!("Binary is ready at: {}", binary_path.display());
    println!("All files in: {}", bin_dir.display());

    // List available files for debugging
    if bin_dir.exists() {
        println!("Available files:");
        for entry in fs::read_dir(bin_dir)? {
            let entry = entry?;
            println!("  - {}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}

/// Find a binary in the given directory by trying multiple possible names
fn find_binary(dir: &Path, names: &[&str]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    for name in names {
        let path = dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(format!(
        "Could not find binary in {}: tried {:?}",
        dir.display(),
        names
    )
    .into())
}
