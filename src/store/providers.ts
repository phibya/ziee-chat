import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { ApiClient } from "../api/client";
import { Model, ModelCapabilities, ModelSettings } from "../types/api/model";
import {
  CreateProviderRequest,
  Provider,
  UpdateProviderRequest,
} from "../types/api/provider";

// Type definitions are now imported from the API types

export interface FileUploadProgress {
  filename: string;
  progress: number;
  status: "pending" | "uploading" | "completed" | "error";
  error?: string;
  size?: number;
}

export interface UploadSession {
  session_id: string;
  total_size_bytes: number;
  main_filename: string;
  provider_id: string;
}

export interface UploadMultipleFilesRequest {
  providerId: string;
  files: File[];
  mainFilename: string;
  name: string;
  alias: string;
  description?: string;
  fileFormat: string;
  capabilities: ModelCapabilities;
  settings?: ModelSettings;
}

interface ProvidersState {
  // Data
  providers: Provider[];
  modelsByProvider: Record<string, Model[]>; // Store models by provider ID

  // Loading states
  loading: boolean;
  creating: boolean;
  updating: boolean;
  deleting: boolean;
  loadingModels: Record<string, boolean>; // Track loading state for provider models
  modelOperations: Record<string, boolean>; // Track loading state for individual models

  // Upload states
  uploading: boolean;
  uploadProgress: FileUploadProgress[];
  overallUploadProgress: number;

  // Upload session state
  uploadSession: UploadSession | null;

  // Error state
  error: string | null;

  // Actions
  loadProviders: () => Promise<void>;
  createProvider: (provider: CreateProviderRequest) => Promise<Provider>;
  updateProvider: (
    id: string,
    provider: UpdateProviderRequest,
  ) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  cloneProvider: (id: string) => Promise<Provider>;

  // Model actions (provider-specific)
  loadProviderModels: (providerId: string) => Promise<void>;
  loadModels: (providerId: string) => Promise<void>; // Alias for compatibility
  addModelToProvider: (
    providerId: string,
    model: {
      name: string;
      alias: string;
      description?: string;
      enabled?: boolean;
      capabilities?: ModelCapabilities;
    },
  ) => Promise<void>;
  addModel: (providerId: string, data: Partial<Model>) => Promise<Model>; // Legacy compatibility
  updateModel: (
    modelId: string,
    updates: { alias?: string; description?: string; enabled?: boolean },
  ) => Promise<void>;
  deleteModel: (modelId: string) => Promise<void>;

  // Upload model actions (for Local) - Upload and auto-commit
  uploadMultipleFilesAndCommit: (
    request: UploadMultipleFilesRequest,
  ) => Promise<Model>;

  // Model control actions (for Local)
  startModel: (modelId: string) => Promise<void>; // For Local
  stopModel: (modelId: string) => Promise<void>; // For Local
  enableModel: (modelId: string) => Promise<void>;
  disableModel: (modelId: string) => Promise<void>;

  // Utility actions
  clearError: () => void;
  cancelUpload: () => void;
  getProviderById: (id: string) => Provider | undefined;
  getModelById: (id: string) => Model | undefined;
}

export const useProvidersStore = create<ProvidersState>()(
  subscribeWithSelector(
    (set, get): ProvidersState => ({
      // Initial state
      providers: [],
      modelsByProvider: {},
      loading: false,
      creating: false,
      updating: false,
      deleting: false,
      loadingModels: {},
      modelOperations: {},
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      uploadSession: null,
      error: null,

      // Actions
      loadProviders: async () => {
        try {
          set({ loading: true, error: null });
          const response = await ApiClient.Providers.list({});
          set({ providers: response.providers, loading: false });
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : "Failed to load providers",
            loading: false,
          });
          throw error;
        }
      },

      createProvider: async (provider: CreateProviderRequest) => {
        try {
          set({ creating: true, error: null });
          const newProvider = await ApiClient.Providers.create(provider);
          set((state) => ({
            providers: [...state.providers, newProvider],
            creating: false,
          }));
          return newProvider;
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : "Failed to create provider",
            creating: false,
          });
          throw error;
        }
      },

      updateProvider: async (id: string, provider: UpdateProviderRequest) => {
        try {
          set({ updating: true, error: null });
          const updatedProvider = await ApiClient.Providers.update({
            provider_id: id,
            ...provider,
          });
          set((state) => ({
            providers: state.providers.map((p) =>
              p.id === id ? updatedProvider : p,
            ),
            updating: false,
          }));
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : "Failed to update provider",
            updating: false,
          });
          throw error;
        }
      },

      deleteProvider: async (id: string) => {
        try {
          set({ deleting: true, error: null });
          await ApiClient.Providers.delete({ provider_id: id });
          set((state) => ({
            providers: state.providers.filter((p) => p.id !== id),
            deleting: false,
          }));
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : "Failed to delete provider",
            deleting: false,
          });
          throw error;
        }
      },

      cloneProvider: async (id: string) => {
        try {
          set({ creating: true, error: null });
          const clonedProvider = await ApiClient.Providers.clone({
            provider_id: id,
          });
          set((state) => ({
            providers: [...state.providers, clonedProvider],
            creating: false,
          }));
          return clonedProvider;
        } catch (error) {
          set({
            error:
              error instanceof Error
                ? error.message
                : "Failed to clone provider",
            creating: false,
          });
          throw error;
        }
      },

      loadProviderModels: async (providerId: string) => {
        try {
          set((state) => ({
            loadingModels: { ...state.loadingModels, [providerId]: true },
            error: null,
          }));
          const models = await ApiClient.Providers.listModels({
            provider_id: providerId,
          });
          set((state) => ({
            modelsByProvider: {
              ...state.modelsByProvider,
              [providerId]: models,
            },
            loadingModels: { ...state.loadingModels, [providerId]: false },
          }));
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to load models",
            loadingModels: { ...state.loadingModels, [providerId]: false },
          }));
          throw error;
        }
      },

      addModelToProvider: async (
        providerId: string,
        model: {
          name: string;
          alias: string;
          description?: string;
          enabled?: boolean;
          capabilities?: ModelCapabilities;
        },
      ) => {
        try {
          set({ creating: true, error: null });
          const newModel = await ApiClient.Providers.addModel({
            provider_id: providerId,
            ...model,
          });
          set((state) => ({
            modelsByProvider: {
              ...state.modelsByProvider,
              [providerId]: [
                ...(state.modelsByProvider[providerId] || []),
                newModel,
              ],
            },
            creating: false,
          }));
        } catch (error) {
          set({
            error:
              error instanceof Error ? error.message : "Failed to add model",
            creating: false,
          });
          throw error;
        }
      },

      updateModel: async (
        modelId: string,
        updates: { alias?: string; description?: string; enabled?: boolean },
      ) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          const updatedModel = await ApiClient.Models.update({
            model_id: modelId,
            ...updates,
          });
          // Update the model in the appropriate provider's list
          set((state) => {
            const newModelsByProvider = { ...state.modelsByProvider };
            for (const providerId in newModelsByProvider) {
              newModelsByProvider[providerId] = newModelsByProvider[
                providerId
              ].map((model) => (model.id === modelId ? updatedModel : model));
            }
            return {
              modelsByProvider: newModelsByProvider,
              modelOperations: { ...state.modelOperations, [modelId]: false },
            };
          });
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to update model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      deleteModel: async (modelId: string) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          await ApiClient.Models.delete({ model_id: modelId });
          // Remove the model from all provider lists
          set((state) => {
            const newModelsByProvider = { ...state.modelsByProvider };
            for (const providerId in newModelsByProvider) {
              newModelsByProvider[providerId] = newModelsByProvider[
                providerId
              ].filter((model) => model.id !== modelId);
            }
            return {
              modelsByProvider: newModelsByProvider,
              modelOperations: { ...state.modelOperations, [modelId]: false },
            };
          });
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to delete model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      startModel: async (modelId: string) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          await ApiClient.Models.start({ model_id: modelId });
          // Reload provider models to get updated state
          const providers = get().providers;
          for (const provider of providers) {
            if (provider.type === "local") {
              await get().loadProviderModels(provider.id);
            }
          }
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to start model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      stopModel: async (modelId: string) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          await ApiClient.Models.stop({ model_id: modelId });
          // Reload provider models to get updated state
          const providers = get().providers;
          for (const provider of providers) {
            if (provider.type === "local") {
              await get().loadProviderModels(provider.id);
            }
          }
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to stop model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      enableModel: async (modelId: string) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          await ApiClient.Models.enable({ model_id: modelId });
          // Update the model status locally
          set((state) => {
            const newModelsByProvider = { ...state.modelsByProvider };
            for (const providerId in newModelsByProvider) {
              newModelsByProvider[providerId] = newModelsByProvider[
                providerId
              ].map((model) =>
                model.id === modelId ? { ...model, enabled: true } : model,
              );
            }
            return {
              modelsByProvider: newModelsByProvider,
              modelOperations: { ...state.modelOperations, [modelId]: false },
            };
          });
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error ? error.message : "Failed to enable model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      disableModel: async (modelId: string) => {
        try {
          set((state) => ({
            modelOperations: { ...state.modelOperations, [modelId]: true },
            error: null,
          }));
          await ApiClient.Models.disable({ model_id: modelId });
          // Update the model status locally
          set((state) => {
            const newModelsByProvider = { ...state.modelsByProvider };
            for (const providerId in newModelsByProvider) {
              newModelsByProvider[providerId] = newModelsByProvider[
                providerId
              ].map((model) =>
                model.id === modelId ? { ...model, enabled: false } : model,
              );
            }
            return {
              modelsByProvider: newModelsByProvider,
              modelOperations: { ...state.modelOperations, [modelId]: false },
            };
          });
        } catch (error) {
          set((state) => ({
            error:
              error instanceof Error
                ? error.message
                : "Failed to disable model",
            modelOperations: { ...state.modelOperations, [modelId]: false },
          }));
          throw error;
        }
      },

      // Upload and auto-commit workflow
      uploadMultipleFilesAndCommit: async (
        request: UploadMultipleFilesRequest,
      ): Promise<Model> => {
        const {
          providerId,
          files,
          mainFilename,
          name,
          alias,
          description,
          fileFormat,
          capabilities,
          settings,
        } = request;

        try {
          set({
            uploading: true,
            error: null,
            uploadProgress: files.map((file) => ({
              filename: file.name,
              progress: 0,
              status: "pending" as const,
              size: file.size,
            })),
            overallUploadProgress: 0,
          });

          // Create multipart form data
          const formData = new FormData();

          // Add all required fields
          formData.append("provider_id", providerId);
          formData.append("main_filename", mainFilename);
          formData.append("name", name);
          formData.append("alias", alias);
          if (description) {
            formData.append("description", description);
          }
          formData.append("file_format", fileFormat);
          if (capabilities) {
            formData.append("capabilities", JSON.stringify(capabilities));
          }
          if (settings) {
            formData.append("settings", JSON.stringify(settings));
          }

          // Add all files
          files.forEach((file) => {
            formData.append("files", file);
          });

          // Upload files and auto-commit
          const model = await ApiClient.ModelUploads.uploadAndCommit(formData, {
            fileUploadProgress: {
              onProgress: (progress, fileIndex, overallProgress) => {
                if (fileIndex !== undefined) {
                  // Update specific file progress
                  set((state) => {
                    const newUploadProgress = [...state.uploadProgress];
                    if (newUploadProgress[fileIndex]) {
                      newUploadProgress[fileIndex] = {
                        ...newUploadProgress[fileIndex],
                        progress,
                        status: progress === 100 ? "completed" : "uploading",
                      };
                    }

                    return {
                      uploadProgress: newUploadProgress,
                      overallUploadProgress: overallProgress || 0,
                    };
                  });
                }
              },
              onError: (error, fileName) => {
                set((state) => ({
                  uploadProgress: state.uploadProgress.map((file) =>
                    fileName && file.filename === fileName
                      ? { ...file, status: "error" as const, error }
                      : file,
                  ),
                  error: error,
                }));
              },
            },
          });

          // Mark all files as completed
          set((state) => ({
            uploadProgress: state.uploadProgress.map((f) => ({
              ...f,
              progress: 100,
              status: "completed" as const,
            })),
            overallUploadProgress: 100,
            uploading: false,
          }));

          // Update models state to include the new model
          set((state) => ({
            modelsByProvider: {
              ...state.modelsByProvider,
              [request.providerId]: [
                ...(state.modelsByProvider[request.providerId] || []),
                model,
              ],
            },
          }));

          // Reload providers to get the latest state
          await get().loadProviders();

          return model;
        } catch (error) {
          set({
            error:
              error instanceof Error ? error.message : "Failed to upload files",
            uploading: false,
            uploadProgress: request.files.map((file) => ({
              filename: file.name,
              progress: 0,
              status: "error" as const,
              error: error instanceof Error ? error.message : "Upload failed",
              size: file.size,
            })),
          });
          throw error;
        }
      },

      clearError: () => set({ error: null }),

      cancelUpload: () => {
        // For now, just reset the upload state
        set({
          uploading: false,
          uploadProgress: [],
          overallUploadProgress: 0,
          error: null,
        });
      },

      // Legacy compatibility functions
      loadModels: async (providerId: string) => {
        // Alias for loadProviderModels for backward compatibility
        return get().loadProviderModels(providerId);
      },

      addModel: async (
        providerId: string,
        data: Partial<Model>,
      ): Promise<Model> => {
        // Legacy compatibility - redirect to addModelToProvider
        await get().addModelToProvider(providerId, {
          name: data.name || "",
          alias: data.alias || "",
          description: data.description,
          enabled: data.enabled,
          capabilities: data.capabilities,
        });

        // Return the newly created model
        const state = get();
        const models = state.modelsByProvider[providerId] || [];
        const newModel = models.find((model) => model.name === data.name);
        if (!newModel) {
          throw new Error("Failed to find newly created model");
        }
        return newModel;
      },

      // Utility functions
      getProviderById: (id: string) => {
        return get().providers.find((provider) => provider.id === id);
      },

      getModelById: (id: string) => {
        const state = get();
        for (const providerId in state.modelsByProvider) {
          const model = state.modelsByProvider[providerId].find(
            (model) => model.id === id,
          );
          if (model) {
            return model;
          }
        }
        return undefined;
      },
    }),
  ),
);
