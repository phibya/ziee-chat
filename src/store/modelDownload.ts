import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { ApiClient } from "../api/client";
import { Model, ModelCapabilities, ModelSettings } from "../types/api/model";

export interface DownloadProgress {
  phase: string;
  current: number;
  total: number;
  message: string;
}

export interface DownloadFromRepositoryRequest {
  provider_id: string;
  repository_id: string;
  repository_path: string;
  main_filename: string;
  repository_branch?: string;
  name: string;
  alias: string;
  description?: string;
  file_format: string;
  capabilities?: ModelCapabilities;
  settings?: ModelSettings;
}

interface ModelDownloadState {
  // Download states
  downloading: boolean;
  downloadProgress: DownloadProgress | null;
  error: string | null;

  // Actions
  downloadFromRepository: (request: DownloadFromRepositoryRequest) => Promise<Model>;
  cancelDownload: () => void;
  clearError: () => void;
}

export const useModelDownloadStore = create<ModelDownloadState>()(
  subscribeWithSelector(
    (set): ModelDownloadState => ({
      // Initial state
      downloading: false,
      downloadProgress: null,
      error: null,

      // Download model from repository with SSE progress tracking
      downloadFromRepository: async (request) => {
        set({
          downloading: true,
          downloadProgress: {
            phase: "Starting",
            current: 0,
            total: 100,
            message: "Initializing repository download...",
          },
          error: null,
        });

        try {
          // biome-ignore lint/suspicious/noAsyncPromiseExecutor: <explanation>
          const model = await new Promise<Model>(async (resolve, reject) => {
            let isRejected = false;
            await ApiClient.Admin.downloadFromRepository(request, {
              SSE: (event: string, data: any) => {
                if (event === "progress") {
                  set({
                    downloadProgress: {
                      phase: data.phase,
                      current: data.current,
                      total: data.total,
                      message: data.message || "Downloading...",
                    },
                  });
                } else if (event === "complete") {
                  set({
                    downloading: false,
                    downloadProgress: null,
                  });

                  const model = data.model as Model;
                  resolve(model);
                } else if (event === "error") {
                  set({
                    downloading: false,
                    downloadProgress: null,
                    error: data.message || "Download failed",
                  });
                  !isRejected &&
                    reject(new Error(data.message || "Download failed"));
                  isRejected = true;
                }
              },
            }).catch((e) => {
              console.error("Download error:", e);
              !isRejected && reject(e);
              isRejected = true;
            });
          });

          return model;
        } catch (error) {
          set({
            downloading: false,
            downloadProgress: null,
            error:
              error instanceof Error
                ? error.message
                : "Failed to download from repository",
          });
          throw error;
        }
      },

      cancelDownload: () => {
        set({
          downloading: false,
          downloadProgress: null,
          error: null,
        });
      },

      clearError: () => set({ error: null }),
    }),
  ),
);