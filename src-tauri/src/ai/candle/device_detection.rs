use crate::database::models::{DeviceInfo, AvailableDevicesResponse};
use std::process::Command;

/// Detect available compute devices on the system
pub fn detect_available_devices() -> AvailableDevicesResponse {
    let mut devices = Vec::new();
    let mut default_device_type = "cpu".to_string();
    let mut supports_multi_gpu = false;

    // Always add CPU device
    devices.push(DeviceInfo {
        id: "cpu".to_string(),
        name: "CPU".to_string(),
        device_type: "cpu".to_string(),
        memory_total: get_system_memory(),
        memory_free: get_available_memory(),
        is_available: true,
    });

    // Check for CUDA devices
    if let Some(cuda_devices) = detect_cuda_devices() {
        devices.extend(cuda_devices);
        if !devices.is_empty() && devices.iter().any(|d| d.device_type == "cuda") {
            default_device_type = "cuda".to_string();
            supports_multi_gpu = devices.iter().filter(|d| d.device_type == "cuda").count() > 1;
        }
    }

    // Check for Metal devices (macOS)
    if let Some(metal_devices) = detect_metal_devices() {
        if !metal_devices.is_empty() {
            default_device_type = "metal".to_string();
        }
        devices.extend(metal_devices);
    }

    AvailableDevicesResponse {
        devices,
        default_device_type,
        supports_multi_gpu,
    }
}

/// Detect CUDA devices using nvidia-smi
fn detect_cuda_devices() -> Option<Vec<DeviceInfo>> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=index,name,uuid,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut devices = Vec::new();

            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    if let (Ok(index), uuid, memory_total, memory_free) = (
                        parts[0].parse::<usize>(),
                        parts[2].to_string(),
                        parts[3].parse::<u64>().ok().map(|m| m * 1024 * 1024), // Convert MB to bytes
                        parts[4].parse::<u64>().ok().map(|m| m * 1024 * 1024), // Convert MB to bytes
                    ) {
                        // Use UUID as device ID if available, otherwise fallback to cuda:index format
                        let device_id = if !uuid.is_empty() && uuid != "N/A" {
                            uuid.clone()
                        } else {
                            format!("cuda:{}", index)
                        };

                        devices.push(DeviceInfo {
                            id: device_id,
                            name: format!("CUDA Device {} ({})", index, parts[1]),
                            device_type: "cuda".to_string(),
                            memory_total,
                            memory_free,
                            is_available: true,
                        });
                    }
                }
            }

            if !devices.is_empty() {
                Some(devices)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Detect Metal devices on macOS
fn detect_metal_devices() -> Option<Vec<DeviceInfo>> {
    #[cfg(target_os = "macos")]
    {
        // Check if we're on macOS with Metal support
        let output = Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-json"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&output_str) {
                    if let Some(displays) = json_data["SPDisplaysDataType"].as_array() {
                        let mut devices = Vec::new();
                        
                        for (index, display) in displays.iter().enumerate() {
                            if let Some(gpu_name) = display["sppci_model"].as_str() {
                                // Estimate memory for Metal devices (this is simplified)
                                let memory_mb = display["spdisplays_vram"]
                                    .as_str()
                                    .and_then(|vram| {
                                        vram.split_whitespace()
                                            .next()
                                            .and_then(|num| num.parse::<u64>().ok())
                                    })
                                    .unwrap_or(8192); // Default to 8GB if can't detect

                                devices.push(DeviceInfo {
                                    id: format!("metal:{}", index),
                                    name: format!("Metal GPU ({})", gpu_name),
                                    device_type: "metal".to_string(),
                                    memory_total: Some(memory_mb * 1024 * 1024), // Convert MB to bytes
                                    memory_free: Some(memory_mb * 1024 * 1024 / 2), // Estimate half available
                                    is_available: true,
                                });
                            }
                        }

                        if !devices.is_empty() {
                            return Some(devices);
                        }
                    }
                }

                // Fallback: Assume Metal is available on macOS
                Some(vec![DeviceInfo {
                    id: "metal:0".to_string(),
                    name: "Metal GPU".to_string(),
                    device_type: "metal".to_string(),
                    memory_total: Some(8 * 1024 * 1024 * 1024), // Default 8GB
                    memory_free: Some(4 * 1024 * 1024 * 1024),  // Estimate 4GB free
                    is_available: true,
                }])
            }
            _ => None,
        }
    }

    #[cfg(not(target_os = "macos"))]
    None
}

/// Get total system memory
fn get_system_memory() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            return output_str.trim().parse::<u64>().ok();
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args(&["computersystem", "get", "TotalPhysicalMemory", "/value"])
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("TotalPhysicalMemory=") {
                    let value = line.trim_start_matches("TotalPhysicalMemory=");
                    return value.parse::<u64>().ok();
                }
            }
        }
    }

    // Fallback estimate
    Some(8 * 1024 * 1024 * 1024) // 8GB default
}

/// Get available system memory
fn get_available_memory() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("vm_stat")
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut free_pages = 0u64;
            let mut page_size = 4096u64; // Default page size
            
            for line in output_str.lines() {
                if line.contains("page size of") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for (i, part) in parts.iter().enumerate() {
                        if part == &"of" && i + 1 < parts.len() {
                            if let Ok(size) = parts[i + 1].parse::<u64>() {
                                page_size = size;
                            }
                        }
                    }
                } else if line.starts_with("Pages free:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let Ok(pages) = parts[2].trim_end_matches('.').parse::<u64>() {
                            free_pages += pages;
                        }
                    }
                }
            }
            
            return Some(free_pages * page_size);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    // Fallback: estimate 50% of total memory is available
    get_system_memory().map(|total| total / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_available_devices() {
        let response = detect_available_devices();
        
        // Should always have at least CPU
        assert!(!response.devices.is_empty());
        assert!(response.devices.iter().any(|d| d.device_type == "cpu"));
        
        // Default device type should be valid
        assert!(["cpu", "cuda", "metal"].contains(&response.default_device_type.as_str()));
    }

    #[test]
    fn test_get_system_memory() {
        let memory = get_system_memory();
        assert!(memory.is_some());
        
        if let Some(mem) = memory {
            // Should be at least 1GB
            assert!(mem >= 1024 * 1024 * 1024);
        }
    }
}