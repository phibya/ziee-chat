export interface HubModel {
  id: string
  name: string
  alias: string
  description?: string
  repository_url: string
  repository_path: string
  main_filename: string
  file_format: string
  capabilities?: {
    vision?: boolean
    audio?: boolean
    tools?: boolean
    code_interpreter?: boolean
  }
  size_gb: number
  tags: string[]
  recommended_parameters?: any
  public: boolean
  popularity_score?: number
  license?: string
  quantization_options?: string[]
  context_length?: number
  language_support?: string[]
}

export interface HubAssistant {
  id: string
  name: string
  description?: string
  instructions?: string
  parameters?: any
  category: string
  tags: string[]
  recommended_models: string[]
  capabilities_required: string[]
  popularity_score?: number
  author?: string
  use_cases?: string[]
  example_prompts?: string[]
}

export interface HubData {
  models: HubModel[]
  assistants: HubAssistant[]
  hub_version: string
  last_updated: string
}

class HubApiClient {
  private baseUrl = '/api/hub'

  async getHubData(): Promise<HubData> {
    const response = await fetch(`${this.baseUrl}/data`)
    if (!response.ok) {
      throw new Error(`Failed to fetch hub data: ${response.statusText}`)
    }
    return response.json()
  }

  async refreshHub(): Promise<HubData> {
    const response = await fetch(`${this.baseUrl}/refresh`, { method: 'POST' })
    if (!response.ok) {
      throw new Error(`Failed to refresh hub: ${response.statusText}`)
    }
    return response.json()
  }

  async getHubVersion(): Promise<string> {
    const response = await fetch(`${this.baseUrl}/version`)
    if (!response.ok) {
      throw new Error(`Failed to get hub version: ${response.statusText}`)
    }
    const data = await response.json()
    return data.hub_version
  }
}

export const hubApiClient = new HubApiClient()
