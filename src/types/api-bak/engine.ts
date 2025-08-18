export interface EngineInfo {
  engine_type: string
  name: string
  version: string
  status: string
  description?: string
  supported_architectures?: string[]
  required_dependencies?: string[]
}

export type EngineListResponse = EngineInfo[]
