use std::collections::HashMap;

/// Platform-specific build configuration
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub name: String,
    pub cmake_flags: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

/// Get platform-specific configuration based on target triple
pub fn get_platform_config(target: &str) -> Result<PlatformConfig, Box<dyn std::error::Error>> {
    if target.contains("darwin") || target.contains("apple") {
        Ok(macos_config(target))
    } else if target.contains("linux") {
        Ok(linux_config(target))
    } else if target.contains("windows") {
        Ok(windows_config(target))
    } else {
        Err(format!("Unsupported target platform: {}", target).into())
    }
}

/// macOS platform configuration
fn macos_config(target: &str) -> PlatformConfig {
    let mut cmake_flags = HashMap::new();
    let env_vars = HashMap::new();

    // Architecture-specific settings
    if target.contains("aarch64") || target.contains("arm64") {
        cmake_flags.insert("CMAKE_OSX_ARCHITECTURES".to_string(), "arm64".to_string());
        // Enable Metal by default on Apple Silicon - use embedded library for simplicity but ensure backends load
        cmake_flags.insert("GGML_METAL".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_USE_BF16".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_EMBED_LIBRARY".to_string(), "ON".to_string());
        cmake_flags.insert("GGML_METAL_SHADER_DEBUG".to_string(), "OFF".to_string());
    } else {
        cmake_flags.insert("CMAKE_OSX_ARCHITECTURES".to_string(), "x86_64".to_string());
        // Disable Metal on Intel Macs due to compatibility issues
        cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());
    }

    // Enable RPC support
    cmake_flags.insert("GGML_RPC".to_string(), "ON".to_string());

    PlatformConfig {
        name: "macOS".to_string(),
        cmake_flags,
        env_vars,
        dependencies: vec!["curl".to_string()],
    }
}

/// Linux platform configuration
fn linux_config(target: &str) -> PlatformConfig {
    let mut cmake_flags = HashMap::new();
    let env_vars = HashMap::new();

    // Architecture-specific settings
    if target.contains("aarch64") || target.contains("arm") {
        // ARM-specific optimizations
        cmake_flags.insert("GGML_CPU_ARM_ARCH".to_string(), "armv8-a".to_string());
    }

    // Enable RPC support
    cmake_flags.insert("GGML_RPC".to_string(), "ON".to_string());

    PlatformConfig {
        name: "Linux".to_string(),
        cmake_flags,
        env_vars,
        dependencies: vec![
            "build-essential".to_string(),
            "libcurl4-openssl-dev".to_string(),
            "libgomp1".to_string(),
        ],
    }
}

/// Windows platform configuration
fn windows_config(target: &str) -> PlatformConfig {
    let mut cmake_flags = HashMap::new();
    let env_vars = HashMap::new();

    // Use Ninja generator for better performance
    cmake_flags.insert(
        "CMAKE_GENERATOR".to_string(),
        "Ninja Multi-Config".to_string(),
    );

    // Architecture-specific settings
    if target.contains("aarch64") || target.contains("arm64") {
        cmake_flags.insert(
            "CMAKE_TOOLCHAIN_FILE".to_string(),
            "cmake/arm64-windows-llvm.cmake".to_string(),
        );
    } else {
        cmake_flags.insert(
            "CMAKE_TOOLCHAIN_FILE".to_string(),
            "cmake/x64-windows-llvm.cmake".to_string(),
        );
    }

    // Windows-specific settings
    // Enable shared libraries for dynamic backend loading (GGML_BACKEND_DL)
    // This allows the same build to work on both GPU and non-GPU machines
    cmake_flags.insert("BUILD_SHARED_LIBS".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_RPC".to_string(), "ON".to_string());
    cmake_flags.insert("LLAMA_BUILD_SERVER".to_string(), "ON".to_string());

    // Linker flags for Windows
    cmake_flags.insert(
        "CMAKE_EXE_LINKER_FLAGS".to_string(),
        "-Wl,--allow-shlib-undefined".to_string(),
    );

    PlatformConfig {
        name: "Windows".to_string(),
        cmake_flags,
        env_vars,
        dependencies: vec!["ninja".to_string(), "curl".to_string()],
    }
}
