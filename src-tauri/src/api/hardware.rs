use axum::{
    debug_handler,
    response::sse::{Event, Sse},
    Extension, Json,
};
use futures_util::stream::Stream;
use schemars::JsonSchema;
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex, time::Duration};
use sysinfo::System;
use tokio::time::interval;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult2,
    middleware::AuthenticatedUser,
};

// Hardware information structures
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OperatingSystemInfo {
    pub name: String,
    pub version: String,
    pub kernel_version: Option<String>,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CPUInfo {
    pub model: String,
    pub architecture: String,
    pub cores: usize,
    pub threads: Option<usize>,
    pub base_frequency: Option<u64>,
    pub max_frequency: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MemoryInfo {
    pub total_ram: u64,
    pub total_swap: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GPUComputeCapabilities {
    pub cuda_support: bool,
    pub cuda_version: Option<String>,
    pub metal_support: bool,
    pub opencl_support: bool,
    pub vulkan_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GPUDevice {
    pub device_id: String, // e.g., "cuda:0", "metal:0", "opencl:0"
    pub name: String,
    pub vendor: String,
    pub memory: Option<u64>,
    pub driver_version: Option<String>,
    pub compute_capabilities: GPUComputeCapabilities,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HardwareInfo {
    pub operating_system: OperatingSystemInfo,
    pub cpu: CPUInfo,
    pub memory: MemoryInfo,
    pub gpu_devices: Vec<GPUDevice>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HardwareInfoResponse {
    pub hardware: HardwareInfo,
}

// Real-time usage structures
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CPUUsage {
    pub usage_percentage: f32,
    pub temperature: Option<f32>,
    pub frequency: Option<u64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MemoryUsage {
    pub used_ram: u64,
    pub available_ram: u64,
    pub used_swap: Option<u64>,
    pub available_swap: Option<u64>,
    pub usage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GPUUsage {
    pub device_id: String, // e.g., "cuda:0", "metal:0", "opencl:0"
    pub device_name: String,
    pub utilization_percentage: Option<f32>,
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    pub memory_usage_percentage: Option<f32>,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HardwareUsageUpdate {
    pub timestamp: String,
    pub cpu: CPUUsage,
    pub memory: MemoryUsage,
    pub gpu_devices: Vec<GPUUsage>,
}

// SSE connection management
type ClientId = Uuid;
lazy_static::lazy_static! {
    static ref SSE_CLIENTS: Mutex<HashMap<ClientId, tokio::sync::mpsc::UnboundedSender<Result<Event, axum::Error>>>> = Mutex::new(HashMap::new());
    static ref MONITORING_ACTIVE: Mutex<bool> = Mutex::new(false);
}

// Get static hardware information
#[debug_handler]
pub async fn get_hardware_info(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Json<HardwareInfoResponse>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get OS information
    let operating_system = OperatingSystemInfo {
        name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        kernel_version: System::kernel_version(),
        architecture: std::env::consts::ARCH.to_string(),
    };

    // Get CPU information
    let cpus = sys.cpus();
    let cpu = CPUInfo {
        model: cpus
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        architecture: std::env::consts::ARCH.to_string(),
        cores: sys.physical_core_count().unwrap_or(cpus.len()),
        threads: Some(cpus.len()),
        base_frequency: cpus.first().map(|cpu| cpu.frequency()),
        max_frequency: None, // sysinfo doesn't provide max frequency directly
    };

    // Get Memory information
    let memory = MemoryInfo {
        total_ram: sys.total_memory(),
        total_swap: Some(sys.total_swap()),
    };

    // Get GPU information with compute capabilities
    let gpu_devices = detect_gpu_devices();

    let hardware_info = HardwareInfo {
        operating_system,
        cpu,
        memory,
        gpu_devices,
    };

    Ok((
        axum::http::StatusCode::OK,
        Json(HardwareInfoResponse {
            hardware: hardware_info,
        }),
    ))
}

// SSE endpoint for real-time hardware usage monitoring
#[debug_handler]
pub async fn subscribe_hardware_usage(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {

    let client_id = Uuid::new_v4();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Add client to the connection pool
    {
        let mut clients = SSE_CLIENTS.lock().unwrap();
        clients.insert(client_id, tx.clone());
    }

    // Send initial connection event
    let _ = tx.send(Ok(Event::default()
        .event("connected")
        .data("{\"message\":\"Hardware monitoring connected\"}")));

    // Start monitoring if not already active
    start_hardware_monitoring().await;

    // Create the SSE stream with proper cleanup
    let stream = async_stream::stream! {
        // Keep the sender alive for the stream lifetime
        let _tx_keeper = tx;

        while let Some(event) = rx.recv().await {
            yield event;
        }

        // Stream ended, remove client
        println!("Hardware monitoring client disconnected: {}", client_id);
        remove_client(client_id);
    };

    Ok((axum::http::StatusCode::OK, Sse::new(stream)))
}

// Start hardware monitoring service
async fn start_hardware_monitoring() {
    let mut monitoring_active = MONITORING_ACTIVE.lock().unwrap();
    if *monitoring_active {
        return; // Already running
    }
    *monitoring_active = true;
    drop(monitoring_active);

    println!("Starting hardware monitoring service");

    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2)); // Update every 2 seconds
        let mut sys = System::new_all();

        loop {
            interval.tick().await;

            // Check if we have any connected clients
            let client_count = {
                let clients = SSE_CLIENTS.lock().unwrap();
                clients.len()
            };

            if client_count == 0 {
                // No clients connected, stop monitoring
                println!("No clients connected, stopping hardware monitoring");
                let mut monitoring_active = MONITORING_ACTIVE.lock().unwrap();
                *monitoring_active = false;
                break;
            }

            // Refresh system information
            sys.refresh_all();

            // Collect usage data
            let usage_update = collect_hardware_usage(&mut sys);

            // Send update to all connected clients
            broadcast_usage_update(usage_update).await;
        }
    });
}

// Collect current hardware usage
fn collect_hardware_usage(sys: &mut System) -> HardwareUsageUpdate {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // CPU usage (average of all cores)
    let cpu_usage = sys.global_cpu_usage();
    let cpu = CPUUsage {
        usage_percentage: cpu_usage,
        temperature: None, // sysinfo doesn't provide CPU temperature on all platforms
        frequency: sys.cpus().first().map(|cpu| cpu.frequency()),
    };

    // Memory usage
    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();
    let available_ram = total_ram - used_ram;
    let usage_percentage = if total_ram > 0 {
        (used_ram as f32 / total_ram as f32) * 100.0
    } else {
        0.0
    };

    let memory = MemoryUsage {
        used_ram,
        available_ram,
        used_swap: Some(sys.used_swap()),
        available_swap: Some(sys.total_swap() - sys.used_swap()),
        usage_percentage,
    };

    // GPU usage (placeholder for now - would need platform-specific implementations)
    let gpu_devices = get_gpu_usage_data();

    HardwareUsageUpdate {
        timestamp,
        cpu,
        memory,
        gpu_devices,
    }
}

// Broadcast usage update to all connected clients
async fn broadcast_usage_update(usage_update: HardwareUsageUpdate) {
    let clients = {
        let clients = SSE_CLIENTS.lock().unwrap();
        clients.clone()
    };

    if clients.is_empty() {
        return;
    }

    let json_data = match serde_json::to_string(&usage_update) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to serialize hardware usage update: {}", e);
            return;
        }
    };

    let event = Event::default().event("update").data(json_data);

    // Send to all clients and remove disconnected ones
    let mut disconnected_clients = Vec::new();

    for (client_id, tx) in clients.iter() {
        if tx.send(Ok(event.clone())).is_err() {
            disconnected_clients.push(*client_id);
        }
    }

    // Remove disconnected clients
    if !disconnected_clients.is_empty() {
        let mut clients = SSE_CLIENTS.lock().unwrap();
        for client_id in disconnected_clients {
            clients.remove(&client_id);
        }
    }
}

// Remove client from connection pool
fn remove_client(client_id: ClientId) {
    let mut clients = SSE_CLIENTS.lock().unwrap();
    clients.remove(&client_id);
    println!("Removed hardware monitoring client: {}", client_id);
}

// Detect GPU devices and their compute capabilities
fn detect_gpu_devices() -> Vec<GPUDevice> {
    let mut gpu_devices = Vec::new();

    // Try to detect GPUs using different methods

    // 1. Try NVIDIA GPUs using NVML
    #[cfg(feature = "gpu-detect")]
    {
        if let Ok(nvidia_gpus) = detect_nvidia_gpus() {
            gpu_devices.extend(nvidia_gpus);
        }
    }

    // 2. Try to detect GPUs using OpenCL (works for AMD, Intel, NVIDIA, Apple)
    #[cfg(feature = "gpu-detect")]
    {
        if let Ok(opencl_gpus) = detect_opencl_gpus() {
            // Only add OpenCL GPUs if we haven't already detected them via NVML
            for opencl_gpu in opencl_gpus {
                if !gpu_devices
                    .iter()
                    .any(|existing| existing.name == opencl_gpu.name)
                {
                    gpu_devices.push(opencl_gpu);
                }
            }
        }
    }

    // 3. Try to detect GPUs using wgpu-hal (cross-platform fallback)
    #[cfg(feature = "gpu-detect")]
    {
        if gpu_devices.is_empty() {
            if let Ok(wgpu_gpus) = detect_wgpu_gpus() {
                gpu_devices.extend(wgpu_gpus);
            }
        }
    }

    // 4. Platform-specific fallbacks if no GPUs detected
    if gpu_devices.is_empty() {
        #[cfg(target_os = "macos")]
        {
            gpu_devices.push(GPUDevice {
                device_id: "metal:0".to_string(),
                name: "Apple GPU".to_string(),
                vendor: "Apple".to_string(),
                memory: None,
                driver_version: None,
                compute_capabilities: GPUComputeCapabilities {
                    cuda_support: false,
                    cuda_version: None,
                    metal_support: true,
                    opencl_support: check_opencl_support(),
                    vulkan_support: Some(check_vulkan_support()),
                },
            });
        }

        #[cfg(not(target_os = "macos"))]
        {
            gpu_devices.push(GPUDevice {
                device_id: "gpu:0".to_string(),
                name: "GPU Device".to_string(),
                vendor: "Unknown".to_string(),
                memory: None,
                driver_version: None,
                compute_capabilities: GPUComputeCapabilities {
                    cuda_support: check_cuda_support(),
                    cuda_version: get_cuda_version(),
                    metal_support: false,
                    opencl_support: check_opencl_support(),
                    vulkan_support: Some(check_vulkan_support()),
                },
            });
        }
    }

    gpu_devices
}

// Get GPU usage data using various methods
fn get_gpu_usage_data() -> Vec<GPUUsage> {
    let mut gpu_usage = Vec::new();

    // Try NVIDIA GPUs first using NVML
    #[cfg(feature = "gpu-detect")]
    {
        if let Ok(nvidia_usage) = get_nvidia_gpu_usage() {
            gpu_usage.extend(nvidia_usage);
        }
    }

    // Add AMD GPU usage detection
    #[cfg(all(feature = "gpu-detect", target_os = "linux"))]
    {
        if let Ok(amd_usage) = get_amd_gpu_usage() {
            gpu_usage.extend(amd_usage);
        }
    }

    // Add Intel GPU usage detection
    #[cfg(feature = "gpu-detect")]
    {
        if let Ok(intel_usage) = get_intel_gpu_usage() {
            gpu_usage.extend(intel_usage);
        }
    }

    // Add Apple GPU usage detection
    #[cfg(all(feature = "gpu-detect", target_os = "macos"))]
    {
        if let Ok(apple_usage) = get_apple_gpu_usage() {
            gpu_usage.extend(apple_usage);
        }
    }

    gpu_usage
}

// NVIDIA GPU detection using NVML with nvidia-smi fallback
#[cfg(feature = "gpu-detect")]
fn detect_nvidia_gpus() -> Result<Vec<GPUDevice>, Box<dyn std::error::Error>> {
    let mut gpu_devices = Vec::new();

    // Try NVML first
    match nvml_wrapper::Nvml::init() {
        Ok(nvml) => {
            if let Ok(device_count) = nvml.device_count() {
                for i in 0..device_count {
                    if let Ok(device) = nvml.device_by_index(i) {
                        let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());
                        let memory = device.memory_info().ok().map(|mem| mem.total);
                        let driver_version = nvml.sys_driver_version().ok();

                        let cuda_version = device
                            .cuda_compute_capability()
                            .ok()
                            .map(|cap| format!("{}.{}", cap.major, cap.minor));

                        gpu_devices.push(GPUDevice {
                            device_id: format!("cuda:{}", i),
                            name,
                            vendor: "NVIDIA".to_string(),
                            memory,
                            driver_version,
                            compute_capabilities: GPUComputeCapabilities {
                                cuda_support: true,
                                cuda_version,
                                metal_support: false,
                                opencl_support: true,
                                vulkan_support: Some(true),
                            },
                        });
                    }
                }
            }
        }
        Err(_) => {
            // NVML failed, try nvidia-smi fallback
            if let Ok(nvidia_gpus) = detect_nvidia_gpus_nvidia_smi() {
                gpu_devices.extend(nvidia_gpus);
            }
        }
    }

    Ok(gpu_devices)
}

// Fallback NVIDIA GPU detection using nvidia-smi
#[cfg(feature = "gpu-detect")]
fn detect_nvidia_gpus_nvidia_smi() -> Result<Vec<GPUDevice>, Box<dyn std::error::Error>> {
    let mut gpu_devices = Vec::new();

    // First, get CUDA version from nvidia-smi header
    let mut cuda_version = None;
    if let Ok(output) = std::process::Command::new("nvidia-smi").output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("CUDA Version:") {
                    if let Some(version_part) = line.split("CUDA Version:").nth(1) {
                        cuda_version = version_part
                            .split_whitespace()
                            .next()
                            .map(|v| v.to_string());
                        break;
                    }
                }
            }
        }
    }

    // Query GPU information
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=index,name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    let index = parts[0].parse::<u32>().unwrap_or(gpu_devices.len() as u32);
                    let name = parts[1].to_string();
                    let memory = parts[2].parse::<u64>().ok().map(|mb| mb * 1024 * 1024);
                    let driver_version = Some(parts[3].to_string());

                    gpu_devices.push(GPUDevice {
                        device_id: format!("cuda:{}", index),
                        name,
                        vendor: "NVIDIA".to_string(),
                        memory,
                        driver_version,
                        compute_capabilities: GPUComputeCapabilities {
                            cuda_support: true,
                            cuda_version: cuda_version.clone(),
                            metal_support: false,
                            opencl_support: true,
                            vulkan_support: Some(true),
                        },
                    });
                }
            }
        }
    }

    Ok(gpu_devices)
}

// Get NVIDIA GPU usage data using NVML
#[cfg(feature = "gpu-detect")]
fn get_nvidia_gpu_usage() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let mut gpu_usage = Vec::new();

    match nvml_wrapper::Nvml::init() {
        Ok(nvml) => {
            let device_count = nvml.device_count()?;

            for i in 0..device_count {
                if let Ok(device) = nvml.device_by_index(i) {
                    let device_name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());

                    let utilization = device.utilization_rates().ok();
                    let memory_info = device.memory_info().ok();
                    let temperature = device
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                        .ok();
                    let power_usage = device.power_usage().ok().map(|p| p as f32 / 1000.0); // Convert mW to W

                    let (memory_usage_percentage, memory_used, memory_total) =
                        if let Some(mem) = memory_info {
                            let percentage = (mem.used as f32 / mem.total as f32) * 100.0;
                            (Some(percentage), Some(mem.used), Some(mem.total))
                        } else {
                            (None, None, None)
                        };

                    gpu_usage.push(GPUUsage {
                        device_id: format!("cuda:{}", i),
                        device_name,
                        utilization_percentage: utilization.map(|u| u.gpu as f32),
                        memory_used,
                        memory_total,
                        memory_usage_percentage,
                        temperature: temperature.map(|t| t as f32),
                        power_usage,
                    });
                }
            }
        }
        Err(_) => {
            // NVML not available
        }
    }

    Ok(gpu_usage)
}

// OpenCL GPU detection (cross-platform)
#[cfg(feature = "gpu-detect")]
fn detect_opencl_gpus() -> Result<Vec<GPUDevice>, Box<dyn std::error::Error>> {
    // For now, return empty result as OpenCL3 API is complex
    // This can be implemented later with proper OpenCL bindings
    Ok(Vec::new())
}

// Simplified GPU detection without wgpu-hal
#[cfg(feature = "gpu-detect")]
fn detect_wgpu_gpus() -> Result<Vec<GPUDevice>, Box<dyn std::error::Error>> {
    // For now, return empty result
    // This can be implemented later with proper wgpu-hal integration
    Ok(Vec::new())
}

// AMD GPU usage detection (Linux only)
#[cfg(all(feature = "gpu-detect", target_os = "linux"))]
fn get_amd_gpu_usage() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let mut gpu_usage = Vec::new();

    // Method 1: Try amdgpu_top with JSON output
    if let Ok(amd_usage) = get_amd_gpu_usage_amdgpu_top() {
        if !amd_usage.is_empty() {
            return Ok(amd_usage);
        }
    }

    // Method 2: Try rocm-smi (ROCm System Management Interface)
    if let Ok(amd_usage) = get_amd_gpu_usage_rocm_smi() {
        if !amd_usage.is_empty() {
            return Ok(amd_usage);
        }
    }

    // Method 3: Fallback to sysfs parsing
    get_amd_gpu_usage_sysfs()
}

// AMD GPU usage detection using amdgpu_top
#[cfg(all(feature = "gpu-detect", target_os = "linux"))]
fn get_amd_gpu_usage_amdgpu_top() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let mut gpu_usage = Vec::new();

    let output = std::process::Command::new("amdgpu_top")
        .args(&["-J", "-i"]) // JSON output, single iteration
        .output()?;

    if !output.status.success() {
        return Ok(gpu_usage);
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
        if let Some(gpus) = json_data.get("gpus").and_then(|v| v.as_array()) {
            for (index, gpu) in gpus.iter().enumerate() {
                let device_name = gpu
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("AMD GPU")
                    .to_string();

                let utilization = gpu
                    .get("gfx_activity")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32);

                let memory_used = gpu.get("vram_usage").and_then(|v| v.as_u64());
                let memory_total = gpu.get("vram_total").and_then(|v| v.as_u64());
                let memory_usage_percentage =
                    if let (Some(used), Some(total)) = (memory_used, memory_total) {
                        Some((used as f32 / total as f32) * 100.0)
                    } else {
                        None
                    };

                let temperature = gpu
                    .get("edge_temperature")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32);

                let power_usage = gpu
                    .get("power_usage")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32);

                gpu_usage.push(GPUUsage {
                    device_id: format!("amd:{}", index),
                    device_name,
                    utilization_percentage: utilization,
                    memory_used,
                    memory_total,
                    memory_usage_percentage,
                    temperature,
                    power_usage,
                });
            }
        }
    }

    Ok(gpu_usage)
}

// AMD GPU usage detection using rocm-smi
#[cfg(all(feature = "gpu-detect", target_os = "linux"))]
fn get_amd_gpu_usage_rocm_smi() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let mut gpu_usage = Vec::new();

    let output = std::process::Command::new("rocm-smi")
        .args(&[
            "--showuse",
            "--showmeminfo",
            "--showtemp",
            "--showpower",
            "--csv",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(gpu_usage);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for (i, line) in output_str.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue; // Skip header and empty lines
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 6 {
            let device_name = format!("AMD GPU {}", i);
            let utilization = parts[1].trim_end_matches('%').parse::<f32>().ok();
            let memory_used = parts[2].parse::<u64>().ok().map(|mb| mb * 1024 * 1024);
            let memory_total = parts[3].parse::<u64>().ok().map(|mb| mb * 1024 * 1024);
            let temperature = parts[4].parse::<f32>().ok();
            let power_usage = parts[5].parse::<f32>().ok();

            let memory_usage_percentage =
                if let (Some(used), Some(total)) = (memory_used, memory_total) {
                    Some((used as f32 / total as f32) * 100.0)
                } else {
                    None
                };

            gpu_usage.push(GPUUsage {
                device_id: format!("amd:{}", i - 1), // i-1 because we skip header line
                device_name,
                utilization_percentage: utilization,
                memory_used,
                memory_total,
                memory_usage_percentage,
                temperature,
                power_usage,
            });
        }
    }

    Ok(gpu_usage)
}

// AMD GPU usage detection using sysfs
#[cfg(all(feature = "gpu-detect", target_os = "linux"))]
fn get_amd_gpu_usage_sysfs() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let mut gpu_usage = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") && !name.contains("-") {
                    let device_path = format!("/sys/class/drm/{}/device", name);

                    // Check if it's AMD (vendor ID 0x1002)
                    if let Ok(vendor) = std::fs::read_to_string(format!("{}/vendor", device_path)) {
                        if vendor.trim() == "0x1002" {
                            let device_name =
                                std::fs::read_to_string(format!("{}/device", device_path))
                                    .ok()
                                    .map(|d| format!("AMD GPU {}", d.trim()))
                                    .unwrap_or_else(|| "AMD GPU".to_string());

                            let utilization = std::fs::read_to_string(format!(
                                "{}/gpu_busy_percent",
                                device_path
                            ))
                            .ok()
                            .and_then(|s| s.trim().parse::<f32>().ok());

                            let memory_used = std::fs::read_to_string(format!(
                                "{}/mem_info_vram_used",
                                device_path
                            ))
                            .ok()
                            .and_then(|s| s.trim().parse::<u64>().ok());

                            let memory_total = std::fs::read_to_string(format!(
                                "{}/mem_info_vram_total",
                                device_path
                            ))
                            .ok()
                            .and_then(|s| s.trim().parse::<u64>().ok());

                            let memory_usage_percentage =
                                if let (Some(used), Some(total)) = (memory_used, memory_total) {
                                    Some((used as f32 / total as f32) * 100.0)
                                } else {
                                    None
                                };

                            gpu_usage.push(GPUUsage {
                                device_id: format!("amd:{}", gpu_usage.len()),
                                device_name,
                                utilization_percentage: utilization,
                                memory_used,
                                memory_total,
                                memory_usage_percentage,
                                temperature: None,
                                power_usage: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(gpu_usage)
}

// Intel GPU usage detection (Linux and Windows)
#[cfg(feature = "gpu-detect")]
fn get_intel_gpu_usage() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let gpu_usage = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Try using intel_gpu_top for Intel GPU monitoring on Linux
        match std::process::Command::new("intel_gpu_top")
            .arg("-J") // JSON output
            .arg("-s") // Single sample
            .arg("1000") // 1 second sample
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let json_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        // Parse Intel GPU usage data from JSON
                        let device_name = "Intel GPU".to_string();

                        let utilization = json_data
                            .get("render/3d")
                            .and_then(|v| v.get("busy"))
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32);

                        gpu_usage.push(GPUUsage {
                            device_id: "intel:0".to_string(),
                            device_name,
                            utilization_percentage: utilization,
                            memory_used: None,
                            memory_total: None,
                            memory_usage_percentage: None,
                            temperature: None,
                            power_usage: None,
                        });
                    }
                }
            }
            Err(_) => {
                // intel_gpu_top not available, try reading from sysfs or other sources
                if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("card") && !name.contains("-") {
                                let device_path = format!("/sys/class/drm/{}/device", name);

                                // Check if this is an Intel GPU
                                if let Ok(vendor) =
                                    std::fs::read_to_string(format!("{}/vendor", device_path))
                                {
                                    if vendor.trim() == "0x8086" {
                                        // Intel vendor ID
                                        let device_name = "Intel GPU".to_string();

                                        // Intel GPUs don't typically expose detailed usage via sysfs
                                        // This would require more complex integration with Intel's tools
                                        gpu_usage.push(GPUUsage {
                                            device_id: format!("intel:{}", gpu_usage.len()),
                                            device_name,
                                            utilization_percentage: None,
                                            memory_used: None,
                                            memory_total: None,
                                            memory_usage_percentage: None,
                                            temperature: None,
                                            power_usage: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, Intel GPU monitoring would require WMI or Performance Counters
        // This is a placeholder for future Windows Intel GPU monitoring implementation
        gpu_usage.push(GPUUsage {
            device_id: "intel:0".to_string(),
            device_name: "Intel GPU".to_string(),
            utilization_percentage: None,
            memory_used: None,
            memory_total: None,
            memory_usage_percentage: None,
            temperature: None,
            power_usage: None,
        });
    }

    Ok(gpu_usage)
}

// Apple GPU usage detection (macOS only)
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_apple_gpu_usage() -> Result<Vec<GPUUsage>, Box<dyn std::error::Error>> {
    let chip_name = get_apple_chip_name();
    let device_name = format!("{} GPU", chip_name);

    // Method 1: Try powermetrics with tasks sampler (doesn't require sudo)
    if let Ok(metrics) = get_apple_gpu_usage_tasks() {
        return Ok(vec![GPUUsage {
            device_id: "metal:0".to_string(),
            device_name,
            utilization_percentage: metrics.utilization,
            memory_used: None, // Apple Silicon uses unified memory
            memory_total: None,
            memory_usage_percentage: None,
            temperature: None, // Temperature reported as pressure level
            power_usage: metrics.power_consumption,
        }]);
    }

    // Method 2: Try powermetrics with gpu_power sampler (may require sudo)
    if let Ok(metrics) = get_apple_gpu_usage_gpu_power() {
        return Ok(vec![GPUUsage {
            device_id: "metal:0".to_string(),
            device_name,
            utilization_percentage: metrics.utilization,
            memory_used: None,
            memory_total: None,
            memory_usage_percentage: None,
            temperature: None,
            power_usage: metrics.power_consumption,
        }]);
    }

    // Method 3: Try activity monitor approach (iokit/IOReport)
    if let Ok(metrics) = get_apple_gpu_usage_iokit() {
        // Calculate memory usage percentage if both values are available
        let memory_usage_percentage =
            if let (Some(used), Some(total)) = (metrics.memory_used, metrics.total_system_memory) {
                if total > 0 {
                    Some((used as f32 / total as f32 * 100.0).min(100.0))
                } else {
                    None
                }
            } else {
                None
            };

        return Ok(vec![GPUUsage {
            device_id: "metal:0".to_string(),
            device_name,
            utilization_percentage: metrics.utilization,
            memory_used: metrics.memory_used,
            memory_total: metrics.total_system_memory,
            memory_usage_percentage,
            temperature: None,
            power_usage: None,
        }]);
    }

    // Fallback: Return basic GPU info without metrics
    Ok(vec![GPUUsage {
        device_id: "metal:0".to_string(),
        device_name,
        utilization_percentage: None,
        memory_used: None,
        memory_total: None,
        memory_usage_percentage: None,
        temperature: None,
        power_usage: None,
    }])
}

// Method 1: Use powermetrics with tasks sampler (no sudo required)
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_apple_gpu_usage_tasks() -> Result<AppleGpuMetrics, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("powermetrics")
        .args(&[
            "--samplers",
            "tasks",
            "--sample-rate",
            "250", // Faster sampling for real-time updates
            "--sample-count",
            "1",
        ])
        .output()?;

    if !output.status.success() {
        return Err("powermetrics tasks failed".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(parse_apple_gpu_metrics_tasks(&output_str))
}

// Method 2: Use powermetrics with gpu_power sampler (may need sudo)
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_apple_gpu_usage_gpu_power() -> Result<AppleGpuMetrics, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("powermetrics")
        .args(&[
            "--samplers",
            "gpu_power",
            "--sample-rate",
            "250",
            "--sample-count",
            "1",
        ])
        .output()?;

    if !output.status.success() {
        return Err("powermetrics gpu_power failed".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(parse_apple_gpu_metrics(&output_str))
}

// Method 3: Use iokit to read GPU usage directly
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_apple_gpu_usage_iokit() -> Result<AppleGpuMetrics, Box<dyn std::error::Error>> {
    // Try using ioreg to get GPU usage from IOKit
    let output = std::process::Command::new("ioreg")
        .args(&["-c", "AGXAccelerator", "-r", "-d1"])
        .output()?;

    if !output.status.success() {
        return Err("ioreg AGXAccelerator failed".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(parse_apple_gpu_metrics_iokit(&output_str))
}

// Parse powermetrics tasks output for GPU utilization
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn parse_apple_gpu_metrics_tasks(output: &str) -> AppleGpuMetrics {
    let mut utilization = None;

    // Look for GPU-related process information in tasks output
    let mut gpu_ms_total = 0.0;
    for line in output.lines() {
        let line = line.trim();

        // Look for GPU time in process entries
        if line.contains("GPU Time") || line.contains("gpu_time") {
            // Extract GPU time in ms/s from process entries
            if let Some(gpu_time_str) = line
                .split("GPU Time")
                .nth(1)
                .or_else(|| line.split("gpu_time").nth(1))
            {
                if let Some(time_str) = gpu_time_str.split("ms/s").next() {
                    if let Ok(gpu_ms) = time_str.trim().parse::<f32>() {
                        gpu_ms_total += gpu_ms;
                    }
                }
            }
        }
    }

    // Calculate GPU utilization as percentage
    // GPU ms/s represents milliseconds of GPU time per second of wall time
    if gpu_ms_total > 0.0 {
        utilization = Some((gpu_ms_total / 10.0).min(100.0)); // Convert ms/s to percentage, cap at 100%
    }

    AppleGpuMetrics {
        utilization,
        power_consumption: None,
        memory_used: None,
        total_system_memory: None,
    }
}

// Parse ioreg output for GPU performance statistics
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn parse_apple_gpu_metrics_iokit(output: &str) -> AppleGpuMetrics {
    let mut utilization = None;
    let mut memory_used = None;

    // Get total system memory
    let total_system_memory = get_system_total_memory();

    // Look for PerformanceStatistics in the ioreg output
    for line in output.lines() {
        let line = line.trim();

        // Look for the PerformanceStatistics line which contains the GPU utilization and memory data
        if line.contains("PerformanceStatistics") {
            // Extract Device Utilization %
            if line.contains("Device Utilization %") {
                if let Some(start) = line.find("\"Device Utilization %\"=") {
                    let after_equals = &line[start + 23..];
                    let mut end_pos = 0;
                    for (i, ch) in after_equals.char_indices() {
                        if ch == ',' || ch == '}' || ch.is_whitespace() {
                            end_pos = i;
                            break;
                        }
                    }
                    if end_pos > 0 {
                        let util_str = &after_equals[..end_pos];
                        utilization = util_str.parse::<f32>().ok();
                    }
                }
            }

            // Extract In use system memory (not the driver version)
            // We want the second occurrence, the one without "(driver)" suffix
            if line.contains("\"In use system memory\"=") {
                // Find the last occurrence to get the non-driver version
                if let Some(start) = line.rfind("\"In use system memory\"=") {
                    let after_equals = &line[start + 23..];
                    let mut end_pos = 0;
                    for (i, ch) in after_equals.char_indices() {
                        if ch == ',' || ch == '}' || ch.is_whitespace() {
                            end_pos = i;
                            break;
                        }
                    }
                    if end_pos > 0 {
                        let mem_str = &after_equals[..end_pos];
                        memory_used = mem_str.parse::<u64>().ok();
                    }
                }
            }

            // Extract Alloc system memory
        }
        // Fallback: Look for standalone device utilization lines (older formats)
        else if line.contains("Device Utilization %") && line.contains("=") {
            if let Some(util_str) = line.split('=').nth(1) {
                let cleaned = util_str.trim().trim_end_matches(',').trim_end_matches('}');
                utilization = cleaned.parse::<f32>().ok();
            }
        }
        // Additional fallback for activity percentages
        else if line.contains("Activity") && line.contains('%') {
            if let Some(percent_pos) = line.find('%') {
                let before_percent = &line[..percent_pos];
                if let Some(last_space) = before_percent.rfind(' ') {
                    let util_str = &before_percent[last_space + 1..];
                    if utilization.is_none() {
                        // Only use as fallback
                        utilization = util_str.parse::<f32>().ok();
                    }
                }
            }
        }
    }

    AppleGpuMetrics {
        utilization,
        power_consumption: None,
        memory_used,
        total_system_memory,
    }
}

// Get total system memory on macOS
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_system_total_memory() -> Option<u64> {
    use std::process::Command;

    let output = Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .ok()?;

    if output.status.success() {
        let mem_str = String::from_utf8_lossy(&output.stdout);
        mem_str.trim().parse::<u64>().ok()
    } else {
        None
    }
}

// Helper functions for GPU capability detection
#[cfg(not(target_os = "macos"))]
fn check_cuda_support() -> bool {
    #[cfg(feature = "gpu-detect")]
    {
        // Check if NVML can initialize (indicates NVIDIA driver presence)
        if let Ok(_nvml) = nvml_wrapper::Nvml::init() {
            return true;
        }
    }

    // Fallback: check for CUDA libraries in system paths
    #[cfg(target_os = "windows")]
    {
        std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA").exists()
    }
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/usr/local/cuda").exists()
            || std::path::Path::new("/opt/cuda").exists()
    }
    #[cfg(target_os = "macos")]
    {
        false // CUDA not supported on modern macOS
    }
}

#[cfg(not(target_os = "macos"))]
fn get_cuda_version() -> Option<String> {
    #[cfg(feature = "gpu-detect")]
    {
        if let Ok(nvml) = nvml_wrapper::Nvml::init() {
            if let Ok(version) = nvml.sys_cuda_driver_version() {
                let major = version / 1000;
                let minor = (version % 1000) / 10;
                return Some(format!("{}.{}", major, minor));
            }
        }
    }
    None
}

fn check_opencl_support() -> bool {
    #[cfg(feature = "gpu-detect")]
    {
        use opencl3::platform::get_platforms;

        // Simple check - if we can get platforms, OpenCL is available
        if let Ok(platforms) = get_platforms() {
            return !platforms.is_empty();
        }
    }
    false
}

fn check_vulkan_support() -> bool {
    #[cfg(feature = "gpu-detect")]
    {
        use ash::vk;

        // Try to create a Vulkan instance
        let entry = match unsafe { ash::Entry::load() } {
            Ok(entry) => entry,
            Err(_) => return false,
        };

        let app_info = vk::ApplicationInfo::default()
            .application_name(std::ffi::CStr::from_bytes_with_nul(b"GPU Detection\0").unwrap())
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

        match unsafe { entry.create_instance(&create_info, None) } {
            Ok(instance) => {
                // Check for physical devices
                let devices = match unsafe { instance.enumerate_physical_devices() } {
                    Ok(devices) => devices,
                    Err(_) => {
                        unsafe { instance.destroy_instance(None) };
                        return false;
                    }
                };

                let has_gpu = !devices.is_empty();
                unsafe { instance.destroy_instance(None) };
                has_gpu
            }
            Err(_) => false,
        }
    }

    #[cfg(not(feature = "gpu-detect"))]
    false
}

// Get Apple chip name using sysctl (faster than system_profiler)
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn get_apple_chip_name() -> String {
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(&["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let cpu_brand = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if cpu_brand.contains("Apple M") {
            for part in cpu_brand.split_whitespace() {
                if part.starts_with("M") && part.chars().nth(1).map_or(false, |c| c.is_numeric()) {
                    let parts: Vec<&str> = cpu_brand.split_whitespace().collect();
                    if let Some(pos) = parts.iter().position(|&x| x == part) {
                        if pos + 1 < parts.len() {
                            let suffix = parts[pos + 1];
                            if suffix == "Pro" || suffix == "Max" || suffix == "Ultra" {
                                return format!("Apple {} {}", part, suffix);
                            }
                        }
                    }
                    return format!("Apple {}", part);
                }
            }
        }
    }
    "Apple Silicon".to_string()
}

// Parse Apple GPU metrics from powermetrics output
#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
fn parse_apple_gpu_metrics(output: &str) -> AppleGpuMetrics {
    let mut utilization = None;
    let mut power_consumption = None;

    for line in output.lines() {
        let line = line.trim();

        if line.contains("GPU HW active residency:") {
            if let Some(percent_str) = line.split(':').nth(1) {
                if let Some(percent) = percent_str.split_whitespace().next() {
                    utilization = percent.trim_end_matches('%').parse::<f32>().ok();
                }
            }
        } else if line.contains("GPU Power:") && !line.contains("CPU + GPU") {
            if let Some(power_str) = line.split(':').nth(1) {
                if let Some(power) = power_str.split_whitespace().next() {
                    power_consumption = power.parse::<f32>().ok().map(|p| p / 1000.0);
                    // Convert mW to W
                }
            }
        }
    }

    AppleGpuMetrics {
        utilization,
        power_consumption,
        memory_used: None,
        total_system_memory: None,
    }
}

#[cfg(all(feature = "gpu-detect", target_os = "macos"))]
struct AppleGpuMetrics {
    utilization: Option<f32>,
    power_consumption: Option<f32>,
    memory_used: Option<u64>,
    total_system_memory: Option<u64>,
}
