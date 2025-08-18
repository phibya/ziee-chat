import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import type { File, FileUploadProgress } from '../../types'
import { ApiClient } from '../../api/client.ts'
import { Message } from '../../types'
import { createStoreProxy } from '../../utils/createStoreProxy.ts'
import { StoreApi, UseBoundStore } from 'zustand/index'
import { useEffect, useMemo } from 'react'
import { useParams } from 'react-router-dom'

// Enable Map and Set support in Immer
enableMapSet()

interface ChatInputUIState {
  content: string
  setContent: (content: string) => void
  // UI state
  isDisabled: boolean
  uploadingFiles: Map<string, FileUploadProgress>
  files: Map<string, File>
  newFiles: Map<string, File>
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
  const store = create<ChatInputUIState>()(
    immer((set, _get, api) => ({
      content: '',
      setContent: (content: string) =>
        set(draft => {
          draft.content = content
        }),
      isDisabled: false,
      uploadingFiles: new Map<string, FileUploadProgress>(),
      files: new Map<string, File>(),
      newFiles: new Map<string, File>(),
      editingMessage: editingMessage,
      destroy: () => {
        //Remove the store from the map and let the browser GC it after the component unmounts
        ChatInputStoreMap.delete(id)
      },
      reset: () => set(api.getState()),
      uploadFiles: async (files: globalThis.File[]) => {
        const newFileProgress = files.map(file => ({
          id: crypto.randomUUID(),
          filename: file.name,
          progress: 0,
          status: 'pending' as const,
          size: file.size,
        }))

        set(draft => {
          newFileProgress.forEach(progress => {
            draft.uploadingFiles.set(progress.id, progress)
          })
        })

        for (let i = 0; i < files.length; i++) {
          const file = files[i]
          const fileProgressId = newFileProgress[i].id

          set(draft => {
            const fileProgress = draft.uploadingFiles.get(fileProgressId)
            if (fileProgress) {
              fileProgress.status = 'uploading'
            }
          })

          const formData = new FormData()
          formData.append('file', file, file.name)

          try {
            const response = await ApiClient.Files.upload(formData, {
              fileUploadProgress: {
                onProgress: (progress: number) => {
                  set(draft => {
                    const fileProgress =
                      draft.uploadingFiles.get(fileProgressId)
                    if (fileProgress) {
                      fileProgress.progress = progress * 100
                    }
                  })
                },
                onComplete: () => {
                  set(draft => {
                    draft.uploadingFiles.delete(fileProgressId)
                  })
                },
                onError: (error: string) => {
                  set(draft => {
                    const fileProgress =
                      draft.uploadingFiles.get(fileProgressId)
                    if (fileProgress) {
                      fileProgress.status = 'error'
                      fileProgress.error = error
                    }
                  })
                },
              },
            })

            set(draft => {
              draft.newFiles.set(response.file.id, response.file)
            })
          } catch (fileError) {
            set(draft => {
              const fileProgress = draft.uploadingFiles.get(fileProgressId)
              if (fileProgress) {
                fileProgress.status = 'error'
                fileProgress.error =
                  fileError instanceof Error
                    ? fileError.message
                    : 'Upload failed'
              }
            })
          }
        }
      },
      removeFile: (fileId: string) => {
        set(draft => {
          draft.files.delete(fileId)
          draft.newFiles.delete(fileId)
        })
      },
      removeUploadingFile: (progressId: string) => {
        set(draft => {
          draft.uploadingFiles.delete(progressId)
        })
      },
    })),
  )

  const storeProxy = createStoreProxy(store)
  ChatInputStoreMap.set(id, storeProxy)
  return storeProxy
}

export const useChatInputUIStore = (editingMessage?: Message) => {
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { projectId } = useParams<{ projectId?: string }>()

  useEffect(() => {
    return () => {
      if (editingMessage?.id) {
        ChatInputStoreMap.delete(editingMessage.id)
      }
    }
  }, [])

  return useMemo(() => {
    const id = editingMessage?.id || conversationId || projectId || 'new-chat'
    return createChatInputUIStore(id, editingMessage)
  }, [])
}
