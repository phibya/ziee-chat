import { Stores } from '../../../../store'
import type { Provider, Model } from '../../../../types'

export const useAvailableModels = () => {
  const { providers, modelsByProvider } = Stores.Providers

  // Get available models grouped by provider, filtered by capability
  const getAvailableModels = (capability?: 'text_embedding' | 'chat') => {
    const options: Array<{
      label: string
      options: Array<{
        label: string
        value: string
        description?: string
      }>
    }> = []

    providers.forEach((provider: Provider) => {
      const providerModels = modelsByProvider[provider.id] || []

      // Filter models by capability if specified
      const filteredModels = capability
        ? providerModels.filter(
            (model: Model) => model.capabilities?.[capability],
          )
        : providerModels

      if (filteredModels.length > 0) {
        options.push({
          label: provider.name,
          options: filteredModels.map((model: Model) => ({
            label: model.alias || model.name,
            value: model.id,
            description: model.description || '',
          })),
        })
      }
    })

    return options
  }

  return {
    getAvailableModels,
    providers,
    modelsByProvider,
  }
}
