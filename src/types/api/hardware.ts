// Hardware API types

// Static hardware information
export interface OperatingSystemInfo {
  name: string
  version: string
  kernel_version?: string
  architecture: string
}

export interface CPUInfo {
  model: string
  architecture: string
  cores: number
  threads?: number
  base_frequency?: number // MHz
  max_frequency?: number // MHz
}

export interface MemoryInfo {
  total_ram: number // bytes
  total_swap?: number // bytes
}

export interface GPUComputeCapabilities {
  cuda_support: boolean
  cuda_version?: string
  metal_support: boolean
  opencl_support: boolean
  vulkan_support?: boolean
}

export interface GPUDevice {
  device_id: string // e.g., "cuda:0", "metal:0", "opencl:0"
  name: string
  vendor: string
  memory?: number // bytes
  driver_version?: string
  compute_capabilities: GPUComputeCapabilities
}

export interface HardwareInfo {
  operating_system: OperatingSystemInfo
  cpu: CPUInfo
  memory: MemoryInfo
  gpu_devices: GPUDevice[]
}

// Real-time usage data
export interface CPUUsage {
  usage_percentage: number
  temperature?: number // Celsius
  frequency?: number // Current frequency in MHz
}

export interface MemoryUsage {
  used_ram: number // bytes
  available_ram: number // bytes
  used_swap?: number // bytes
  available_swap?: number // bytes
  usage_percentage: number
}

export interface GPUUsage {
  device_id: string // e.g., "cuda:0", "metal:0", "opencl:0"
  device_name: string
  utilization_percentage?: number
  memory_used?: number // bytes
  memory_total?: number // bytes
  memory_usage_percentage?: number
  temperature?: number // Celsius
  power_usage?: number // Watts
}

export interface HardwareUsageUpdate {
  timestamp: string
  cpu: CPUUsage
  memory: MemoryUsage
  gpu_devices: GPUUsage[]
}

// SSE Events for hardware usage monitoring
export interface HardwareUsageSSEEvent {
  event: 'update' | 'connected' | 'disconnected' | 'error'
  data: HardwareUsageUpdate | { message: string } | { error: string }
}

// API Response types
export interface HardwareInfoResponse {
  hardware: HardwareInfo
}
