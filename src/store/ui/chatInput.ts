import { create } from 'zustand'
import type { File, FileUploadProgress } from '../../types'
import { ApiClient } from '../../api/client.ts'
import { Message } from '../../types/api/chat.ts'
import { createStoreProxy } from '../../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'

interface ChatInputUIState {
  content: string
  setContent: (content: string) => void
  // UI state
  isDisabled: boolean
  uploadingFiles: FileUploadProgress[]
  files: File[]
  newFiles: File[]
  // Editing state
  editingMessage: Message | undefined
  destroy: () => void
  // methods
  reset: () => void
  uploadFiles: (files: globalThis.File[]) => Promise<void>
  removeFile: (fileId: string) => void
  removeUploadingFile: (progressId: string) => void
}

const ChatInputStoreMap = new Map<
  string,
  ReturnType<typeof createStoreProxy<UseBoundStore<StoreApi<ChatInputUIState>>>>
>()

export const createChatInputUIStore = (
  id: string,
  editingMessage?: Message,
) => {
  if (ChatInputStoreMap.has(id)) {
    return ChatInputStoreMap.get(id)!
  }
  const storeProxy = createStoreProxy(
    create<ChatInputUIState>((set, _get, store) => ({
      content: '',
      setContent: (content: string) => set({ content }),
      isDisabled: false,
      uploadingFiles: [],
      files: [],
      newFiles: [],
      editingMessage: editingMessage,
      destroy: () => {
        //Remove the store from the map and let the browser GC it after the component unmounts
        ChatInputStoreMap.delete(id)
      },
      reset: () => set(store.getState()),
      uploadFiles: async (files: globalThis.File[]) => {
        const newFileProgress = files.map(file => ({
          id: crypto.randomUUID(),
          filename: file.name,
          progress: 0,
          status: 'pending' as const,
          size: file.size,
        }))

        set(state => ({
          uploadingFiles: [...state.uploadingFiles, ...newFileProgress],
        }))

        for (let i = 0; i < files.length; i++) {
          const file = files[i]
          const fileProgressId = newFileProgress[i].id

          set(state => ({
            uploadingFiles: state.uploadingFiles.map(fp =>
              fp.id === fileProgressId
                ? { ...fp, status: 'uploading' as const }
                : fp,
            ),
          }))

          const formData = new FormData()
          formData.append('file', file, file.name)

          try {
            const response = await ApiClient.Files.upload(formData, {
              fileUploadProgress: {
                onProgress: (progress: number) => {
                  set(state => ({
                    uploadingFiles: state.uploadingFiles.map(fp =>
                      fp.id === fileProgressId
                        ? { ...fp, progress: progress * 100 }
                        : fp,
                    ),
                  }))
                },
                onComplete: () => {
                  set(state => ({
                    uploadingFiles: state.uploadingFiles.filter(
                      fp => fp.id !== fileProgressId,
                    ),
                  }))
                },
                onError: (error: string) => {
                  set(state => ({
                    uploadingFiles: state.uploadingFiles.map(fp =>
                      fp.id === fileProgressId
                        ? { ...fp, status: 'error' as const, error }
                        : fp,
                    ),
                  }))
                },
              },
            })

            set(state => ({
              newFiles: [...state.newFiles, response.file],
            }))
          } catch (fileError) {
            set(state => ({
              uploadingFiles: state.uploadingFiles.map(fp =>
                fp.id === fileProgressId
                  ? {
                      ...fp,
                      status: 'error' as const,
                      error:
                        fileError instanceof Error
                          ? fileError.message
                          : 'Upload failed',
                    }
                  : fp,
              ),
            }))
          }
        }
      },
      removeFile: (fileId: string) => {
        set(state => ({
          files: state.files.filter(file => file.id !== fileId),
          newFiles: state.newFiles.filter(file => file.id !== fileId),
        }))
      },
      removeUploadingFile: (progressId: string) => {
        set(state => ({
          uploadingFiles: state.uploadingFiles.filter(
            fp => fp.id !== progressId,
          ),
        }))
      },
    })),
  )

  ChatInputStoreMap.set(id, storeProxy)
  return storeProxy
}
