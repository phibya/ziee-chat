import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { ApiClient } from "../api/client";
import { Model, ModelCapabilities, ModelSettings } from "../types/api/model";
import { loadModelsForProvider } from "./providers";

export interface FileUploadProgress {
  filename: string;
  progress: number;
  status: "pending" | "uploading" | "completed" | "error";
  error?: string;
  size?: number;
}

export interface LocalUploadRequest {
  provider_id: string;
  files: File[];
  main_filename: string;
  name: string;
  alias: string;
  description?: string;
  file_format: string;
  capabilities: ModelCapabilities;
  settings?: ModelSettings;
}

interface LocalUploadState {
  // Upload state
  uploading: boolean;
  uploadProgress: FileUploadProgress[];
  overallUploadProgress: number;

  // Error state
  error: string | null;

  // UI state
  showProgress: boolean;
}

export const useLocalUploadStore = create<LocalUploadState>()(
  subscribeWithSelector(
    (): LocalUploadState => ({
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      error: null,
      showProgress: false,
    }),
  ),
);

// Upload actions
export const uploadLocalModel = async (
  request: LocalUploadRequest,
): Promise<Model> => {
  try {
    useLocalUploadStore.setState({
      uploading: true,
      uploadProgress: request.files.map((file) => ({
        filename: file.name,
        progress: 0,
        status: "pending" as const,
        size: file.size,
      })),
      overallUploadProgress: 0,
      error: null,
      showProgress: true,
    });

    // Create FormData for the multipart request
    const formData = new FormData();

    // Add files to FormData
    request.files.forEach((file) => {
      formData.append("files", file);
    });

    // Add metadata fields
    formData.append("provider_id", request.provider_id);
    formData.append("name", request.name);
    formData.append("alias", request.alias);
    formData.append("main_filename", request.main_filename);
    formData.append("file_format", request.file_format);

    if (request.description) {
      formData.append("description", request.description);
    }

    if (request.capabilities) {
      formData.append("capabilities", JSON.stringify(request.capabilities));
    }

    if (request.settings) {
      formData.append("settings", JSON.stringify(request.settings));
    }

    // Call the upload API with file upload progress tracking
    const model = await ApiClient.ModelUploads.uploadAndCommit(formData, {
      fileUploadProgress: {
        onProgress: (
          progress: number,
          fileIndex: number,
          overallProgress: number,
        ) => {
          // Handle file-specific upload progress
          useLocalUploadStore.setState((state) => ({
            uploadProgress: state.uploadProgress.map((fp, index) =>
              index === fileIndex
                ? {
                    ...fp,
                    progress: progress,
                    status:
                      progress >= 1
                        ? ("completed" as const)
                        : ("uploading" as const),
                  }
                : fp,
            ),
            overallUploadProgress: overallProgress,
          }));
        },
        onComplete: () => {
          // Handle upload completion
          useLocalUploadStore.setState((state) => ({
            uploadProgress: state.uploadProgress.map((fp) => ({
              ...fp,
              progress: 100,
              status: "completed" as const,
            })),
            overallUploadProgress: 100,
            uploading: false,
            showProgress: false,
          }));

          // Refresh the provider's models list
          loadModelsForProvider(request.provider_id);
        },
        onError: (error: string, fileName?: string) => {
          // Handle upload error
          useLocalUploadStore.setState((state) => ({
            uploadProgress: state.uploadProgress.map((fp) =>
              fileName && fp.filename === fileName
                ? { ...fp, status: "error" as const, error }
                : fp,
            ),
            error: error || "Upload failed",
            uploading: false,
            showProgress: true,
          }));
        },
      },
    });

    return model;
  } catch (error) {
    useLocalUploadStore.setState({
      error: error instanceof Error ? error.message : "Failed to upload model",
      uploading: false,
      uploadProgress: [],
      overallUploadProgress: 0,
      showProgress: false,
    });
    throw error;
  }
};

// Utility actions
export const cancelLocalUpload = (): void => {
  useLocalUploadStore.setState({
    uploading: false,
    uploadProgress: [],
    overallUploadProgress: 0,
    showProgress: false,
  });
};

export const clearLocalUploadError = (): void => {
  useLocalUploadStore.setState({ error: null });
};

export const hideUploadProgress = (): void => {
  useLocalUploadStore.setState({ showProgress: false });
};

export const showUploadProgress = (): void => {
  useLocalUploadStore.setState({ showProgress: true });
};
