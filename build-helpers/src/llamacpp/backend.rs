use std::env;
use std::collections::HashMap;

/// Available backend types for llama.cpp
#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    CPU,
    CUDA,
    Metal,
    Vulkan,
    OpenCL,
    BLAS,
    SYCL,
    HIP,
    MUSA,
}

/// Off by default; enable with ZIEE_USE_GPU=1|true|on
pub fn wants_gpu() -> bool {
    matches!(env::var("ZIEE_USE_GPU").as_deref(), Ok("1") | Ok("true") | Ok("on"))
}

/// Portable default backend by target when GPU requested
pub fn default_backend_for_target(target: &str) -> BackendType {
    if !wants_gpu() {
        return BackendType::CPU;
    }
    if target.contains("darwin") || target.contains("apple") {
        BackendType::Metal
    } else if target.contains("windows") || target.contains("linux") {
        BackendType::CUDA
    } else {
        BackendType::CPU
    }
}

impl BackendType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "cpu" => Ok(BackendType::CPU),
            "cuda" => Ok(BackendType::CUDA),
            "metal" => Ok(BackendType::Metal),
            "vulkan" => Ok(BackendType::Vulkan),
            "opencl" => Ok(BackendType::OpenCL),
            "blas" | "openblas" => Ok(BackendType::BLAS),
            "sycl" => Ok(BackendType::SYCL),
            "hip" | "rocm" => Ok(BackendType::HIP),
            "musa" => Ok(BackendType::MUSA),
            _ => Err(format!("Unknown backend type: {}", s)),
        }
    }
}

/// Backend-specific build configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub name: String,
    pub backend_type: BackendType,
    pub cmake_flags: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

/// Get backend-specific configuration
pub fn get_backend_config(
    backend: BackendType,
    target: &str,
) -> Result<BackendConfig, Box<dyn std::error::Error>> {
    match backend {
        BackendType::CPU => Ok(cpu_config()),
        BackendType::CUDA => Ok(cuda_config()),
        BackendType::Metal => Ok(metal_config(target)?),
        BackendType::Vulkan => Ok(vulkan_config()),
        BackendType::OpenCL => Ok(opencl_config()),
        BackendType::BLAS => Ok(blas_config(target)),
        BackendType::SYCL => Ok(sycl_config()),
        BackendType::HIP => Ok(hip_config()),
        BackendType::MUSA => Ok(musa_config()),
    }
}

/// Get comprehensive multi-backend configuration for Windows/Linux with all features
pub fn get_multi_backend_config(target: &str) -> Result<BackendConfig, Box<dyn std::error::Error>> {
    // Only enable multi-backend on Windows and Linux
    if target.contains("darwin") || target.contains("apple") {
        return Err(
            "Multi-backend configuration is not supported on macOS (use Metal instead)".into(),
        );
    }

    let mut cmake_flags = HashMap::new();
    let mut dependencies = vec![];

    // Enable CPU backend with all optimizations
    cmake_flags.insert("GGML_CPU".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());

    // Enable all SIMD optimizations for maximum compatibility
    cmake_flags.insert("GGML_AVX2".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_AVX".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_SSE3".to_string(), "ON".to_string());

    // Enable dynamic backend loading
    cmake_flags.insert("GGML_BACKEND_DL".to_string(), "OFF".to_string());

    // Enable CUDA support with comprehensive architecture coverage
    cmake_flags.insert("GGML_CUDA".to_string(), "ON".to_string());
    cmake_flags.insert(
        "CMAKE_CUDA_ARCHITECTURES".to_string(),
        "52;61;70;75;80;86;89;90".to_string(),
    );
    dependencies.push("nvidia-cuda-toolkit".to_string());

    // Enable Vulkan support
    cmake_flags.insert("GGML_VULKAN".to_string(), "ON".to_string());
    dependencies.push("vulkan-sdk".to_string());
    if target.contains("linux") {
        dependencies.push("mesa-vulkan-drivers".to_string());
    }

    // Enable OpenCL support
    cmake_flags.insert("GGML_OPENCL".to_string(), "ON".to_string());
    dependencies.push("opencl-headers".to_string());
    if target.contains("linux") {
        dependencies.push("ocl-icd-libopencl1".to_string());
    }

    // Disable macOS-specific backends
    cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());

    Ok(BackendConfig {
        name: "Comprehensive (CUDA+Vulkan+OpenCL+CPU)".to_string(),
        backend_type: BackendType::CUDA, // Use CUDA as primary type
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies,
    })
}

/// CPU backend configuration - enables base CPU support
fn cpu_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable CPU backend specifically
    cmake_flags.insert("GGML_CPU".to_string(), "ON".to_string());

    // CPU optimizations
    cmake_flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "OFF".to_string());

    // Note: Don't disable other backends here since this might be used in combination
    // with Metal/CUDA. The other backend configs will override these as needed.

    BackendConfig {
        name: "CPU".to_string(),
        backend_type: BackendType::CPU,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec![],
    }
}

/// CUDA backend configuration
fn cuda_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();
    let env_vars = HashMap::new();

    // Enable CUDA
    cmake_flags.insert("GGML_CUDA".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_BACKEND_DL".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());

    // CUDA architecture settings (default to modern GPUs)
    cmake_flags.insert(
        "CMAKE_CUDA_ARCHITECTURES".to_string(),
        "70;75;80;86;89".to_string(),
    );

    // Only disable Metal on non-macOS platforms (Vulkan and OpenCL can coexist with CUDA)
    cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());

    BackendConfig {
        name: "CUDA".to_string(),
        backend_type: BackendType::CUDA,
        cmake_flags,
        env_vars,
        dependencies: vec!["nvidia-cuda-toolkit".to_string()],
    }
}

/// Metal backend configuration (macOS only)
fn metal_config(target: &str) -> Result<BackendConfig, Box<dyn std::error::Error>> {
    if !target.contains("darwin") && !target.contains("apple") {
        return Err("Metal backend is only available on macOS".into());
    }

    let mut cmake_flags = HashMap::new();

    // Enable Metal with embedded library and ensure backends load properly
    cmake_flags.insert("GGML_METAL".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_METAL_USE_BF16".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_METAL_EMBED_LIBRARY".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_METAL_SHADER_DEBUG".to_string(), "OFF".to_string());

    // Disable other GPU backends
    cmake_flags.insert("GGML_CUDA".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_VULKAN".to_string(), "OFF".to_string());

    Ok(BackendConfig {
        name: "Metal".to_string(),
        backend_type: BackendType::Metal,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec![],
    })
}

/// Vulkan backend configuration
fn vulkan_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable Vulkan
    cmake_flags.insert("GGML_VULKAN".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_BACKEND_DL".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "OFF".to_string());

    // Only disable Metal (CUDA and OpenCL can coexist with Vulkan)
    cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());

    BackendConfig {
        name: "Vulkan".to_string(),
        backend_type: BackendType::Vulkan,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec!["vulkan-sdk".to_string(), "mesa-vulkan-drivers".to_string()],
    }
}

/// OpenCL backend configuration
fn opencl_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable OpenCL
    cmake_flags.insert("GGML_OPENCL".to_string(), "ON".to_string());

    // Only disable Metal (CUDA and Vulkan can coexist with OpenCL)
    cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());

    BackendConfig {
        name: "OpenCL".to_string(),
        backend_type: BackendType::OpenCL,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec![
            "opencl-headers".to_string(),
            "ocl-icd-libopencl1".to_string(),
        ],
    }
}

/// BLAS backend configuration (platform-specific)
fn blas_config(target: &str) -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable BLAS
    cmake_flags.insert("GGML_BLAS".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_OPENMP".to_string(), "OFF".to_string());

    let (vendor, dependencies) = if target.contains("darwin") || target.contains("apple") {
        // macOS: Use built-in Accelerate framework
        ("Apple".to_string(), vec![])
    } else {
        // Linux/Windows: Use OpenBLAS
        ("OpenBLAS".to_string(), vec!["libopenblas-dev".to_string()])
    };

    cmake_flags.insert("GGML_BLAS_VENDOR".to_string(), vendor);

    // Disable GPU backends (except ones that should be enabled)
    cmake_flags.insert("GGML_CUDA".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_VULKAN".to_string(), "OFF".to_string());

    BackendConfig {
        name: "BLAS".to_string(),
        backend_type: BackendType::BLAS,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies,
    }
}

/// SYCL backend configuration (Intel oneAPI)
fn sycl_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();
    let mut env_vars = HashMap::new();

    // Enable SYCL
    cmake_flags.insert("GGML_SYCL".to_string(), "ON".to_string());
    cmake_flags.insert("CMAKE_C_COMPILER".to_string(), "icx".to_string());
    cmake_flags.insert("CMAKE_CXX_COMPILER".to_string(), "icpx".to_string());

    // Intel oneAPI environment
    env_vars.insert("ONEAPI_ROOT".to_string(), "/opt/intel/oneapi".to_string());

    BackendConfig {
        name: "SYCL".to_string(),
        backend_type: BackendType::SYCL,
        cmake_flags,
        env_vars,
        dependencies: vec![
            "intel-oneapi-compiler-dpcpp-cpp".to_string(),
            "intel-oneapi-mkl-devel".to_string(),
        ],
    }
}

/// HIP backend configuration (AMD ROCm)
fn hip_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable HIP
    cmake_flags.insert("GGML_HIP".to_string(), "ON".to_string());
    cmake_flags.insert("GGML_HIP_ROCWMMA_FATTN".to_string(), "ON".to_string());

    // Disable other GPU backends
    cmake_flags.insert("GGML_CUDA".to_string(), "OFF".to_string());
    cmake_flags.insert("GGML_METAL".to_string(), "OFF".to_string());

    BackendConfig {
        name: "HIP".to_string(),
        backend_type: BackendType::HIP,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec!["rocblas-dev".to_string(), "hipblas-dev".to_string()],
    }
}

/// MUSA backend configuration
fn musa_config() -> BackendConfig {
    let mut cmake_flags = HashMap::new();

    // Enable MUSA
    cmake_flags.insert("GGML_MUSA".to_string(), "ON".to_string());

    BackendConfig {
        name: "MUSA".to_string(),
        backend_type: BackendType::MUSA,
        cmake_flags,
        env_vars: HashMap::new(),
        dependencies: vec![],
    }
}
