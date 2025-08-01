import { create } from 'zustand'
import type { File, FileUploadProgress } from '../../types'
import { ApiClient } from '../../api/client.ts'

interface ChatUIState {
  // Message editing state
  editingMessageId: string | null
  editingMessageContent: string
  showMessageToolBox: { [messageId: string]: boolean }

  // Chat input state
  inputDisabled: boolean
  inputPlaceholder: string
  //UIInterface can be new-chat, new-chat-in-project, or {conversationId}
  fileUploadProgressByUIInterface: Record<string, FileUploadProgress[]>
  filesByUIInterface: Record<string, File[]>
}

export const useChatUIStore = create<ChatUIState>(() => ({
  // Initial state
  editingMessageId: null,
  editingMessageContent: '',
  showMessageToolBox: {},
  inputDisabled: false,
  inputPlaceholder: '',
  fileUploadProgressByUIInterface: {},
  filesByUIInterface: {},
}))

// Actions
export const startEditingMessage = (messageId: string, content: string) => {
  useChatUIStore.setState({
    editingMessageId: messageId,
    editingMessageContent: content,
  })
}

export const stopEditingMessage = () => {
  useChatUIStore.setState({
    editingMessageId: null,
    editingMessageContent: '',
  })
}

export const updateEditingContent = (content: string) => {
  useChatUIStore.setState({
    editingMessageContent: content,
  })
}

export const setMessageToolBoxVisible = (
  messageId: string,
  visible: boolean,
) => {
  useChatUIStore.setState(state => ({
    showMessageToolBox: {
      ...state.showMessageToolBox,
      [messageId]: visible,
    },
  }))
}

export const setInputDisabled = (disabled: boolean) => {
  useChatUIStore.setState({ inputDisabled: disabled })
}

export const setInputPlaceholder = (placeholder: string) => {
  useChatUIStore.setState({ inputPlaceholder: placeholder })
}

export const resetChatUI = (uiInterface?: string) => {
  if (uiInterface) {
    // Reset specific UI interface
    useChatUIStore.setState(state => ({
      fileUploadProgressByUIInterface: {
        ...state.fileUploadProgressByUIInterface,
        [uiInterface]: [],
      },
      filesByUIInterface: {
        ...state.filesByUIInterface,
        [uiInterface]: [],
      },
    }))
  } else {
    // Reset all UI state
    useChatUIStore.setState({
      editingMessageId: null,
      editingMessageContent: '',
      showMessageToolBox: {},
      inputDisabled: false,
      inputPlaceholder: '',
      fileUploadProgressByUIInterface: {},
      filesByUIInterface: {},
    })
  }
}

// File upload actions
export const uploadFilesToChat = async (
  uiInterface: string,
  files: globalThis.File[],
): Promise<File[]> => {
  try {
    // Initialize upload progress for specific UI interface (append to existing)
    const newFileProgress = files.map(file => ({
      id: crypto.randomUUID(),
      filename: file.name,
      progress: 0,
      status: 'pending' as const,
      size: file.size,
    }))

    useChatUIStore.setState(state => ({
      fileUploadProgressByUIInterface: {
        ...state.fileUploadProgressByUIInterface,
        [uiInterface]: [
          ...(state.fileUploadProgressByUIInterface[uiInterface] || []),
          ...newFileProgress,
        ],
      },
    }))

    const uploadedFiles: File[] = []

    // Upload files sequentially to better track progress
    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      const fileProgressId = newFileProgress[i].id

      // Update current file status to uploading
      useChatUIStore.setState(state => ({
        fileUploadProgressByUIInterface: {
          ...state.fileUploadProgressByUIInterface,
          [uiInterface]: (
            state.fileUploadProgressByUIInterface[uiInterface] || []
          ).map((fp: FileUploadProgress) =>
            fp.id === fileProgressId
              ? { ...fp, status: 'uploading' as const }
              : fp,
          ),
        },
      }))

      // Create FormData for the file
      const formData = new FormData()
      formData.append('file', file, file.name)

      try {
        // Call the upload API with progress tracking using ApiClient.Files.upload
        const response = await ApiClient.Files.upload(formData, {
          fileUploadProgress: {
            onProgress: (progress: number) => {
              // Update file-specific progress
              useChatUIStore.setState(state => ({
                fileUploadProgressByUIInterface: {
                  ...state.fileUploadProgressByUIInterface,
                  [uiInterface]: (
                    state.fileUploadProgressByUIInterface[uiInterface] || []
                  ).map((fp: FileUploadProgress) =>
                    fp.id === fileProgressId
                      ? { ...fp, progress: progress * 100 }
                      : fp,
                  ),
                },
              }))
            },
            onComplete: () => {
              // Remove completed file from upload progress
              useChatUIStore.setState(state => ({
                fileUploadProgressByUIInterface: {
                  ...state.fileUploadProgressByUIInterface,
                  [uiInterface]: (
                    state.fileUploadProgressByUIInterface[uiInterface] || []
                  ).filter(
                    (fp: FileUploadProgress) => fp.id !== fileProgressId,
                  ),
                },
              }))
            },
            onError: (error: string) => {
              // Mark this file as failed
              useChatUIStore.setState(state => ({
                fileUploadProgressByUIInterface: {
                  ...state.fileUploadProgressByUIInterface,
                  [uiInterface]: (
                    state.fileUploadProgressByUIInterface[uiInterface] || []
                  ).map((fp: FileUploadProgress) =>
                    fp.id === fileProgressId
                      ? {
                          ...fp,
                          status: 'error' as const,
                          error: error,
                        }
                      : fp,
                  ),
                },
              }))
            },
          },
        })

        uploadedFiles.push(response.file)
      } catch (fileError) {
        // Mark this file as failed
        useChatUIStore.setState(state => ({
          fileUploadProgressByUIInterface: {
            ...state.fileUploadProgressByUIInterface,
            [uiInterface]: (
              state.fileUploadProgressByUIInterface[uiInterface] || []
            ).map((fp: FileUploadProgress) =>
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
          },
        }))
      }
    }

    // Add uploaded files to the chat files list for specific UI interface
    useChatUIStore.setState(state => ({
      filesByUIInterface: {
        ...state.filesByUIInterface,
        [uiInterface]: [
          ...(state.filesByUIInterface[uiInterface] || []),
          ...uploadedFiles,
        ],
      },
    }))

    // Note: Completed files are automatically removed from progress on completion
    // Only failed files remain in progress for user to see errors

    return uploadedFiles
  } catch (error) {
    useChatUIStore.setState(state => ({
      fileUploadProgressByUIInterface: {
        ...state.fileUploadProgressByUIInterface,
        [uiInterface]: [],
      },
    }))
    throw error
  }
}

export const removeFileFromChat = (uiInterface: string, fileId: string) => {
  useChatUIStore.setState(state => ({
    filesByUIInterface: {
      ...state.filesByUIInterface,
      [uiInterface]: (state.filesByUIInterface[uiInterface] || []).filter(
        (file: File) => file.id !== fileId,
      ),
    },
  }))
}

export const removeFileUploadProgress = (
  uiInterface: string,
  progressId: string,
) => {
  useChatUIStore.setState(state => ({
    fileUploadProgressByUIInterface: {
      ...state.fileUploadProgressByUIInterface,
      [uiInterface]: (
        state.fileUploadProgressByUIInterface[uiInterface] || []
      ).filter((fp: FileUploadProgress) => fp.id !== progressId),
    },
  }))
}

export const clearChatFiles = (uiInterface: string) => {
  useChatUIStore.setState(state => ({
    filesByUIInterface: {
      ...state.filesByUIInterface,
      [uiInterface]: [],
    },
    fileUploadProgressByUIInterface: {
      ...state.fileUploadProgressByUIInterface,
      [uiInterface]: [],
    },
  }))
}

export const cancelChatFileUpload = (uiInterface: string) => {
  useChatUIStore.setState(state => ({
    fileUploadProgressByUIInterface: {
      ...state.fileUploadProgressByUIInterface,
      [uiInterface]: [],
    },
  }))
}

// Helper functions to get interface-specific data
export const getFileUploadProgress = (
  uiInterface: string,
): FileUploadProgress[] => {
  const state = useChatUIStore.getState()
  return state.fileUploadProgressByUIInterface[uiInterface] || []
}

export const getChatFiles = (uiInterface: string): File[] => {
  const state = useChatUIStore.getState()
  return state.filesByUIInterface[uiInterface] || []
}

export const getFileUploadProgressByUIInterface = (): Record<
  string,
  FileUploadProgress[]
> => {
  const state = useChatUIStore.getState()
  return state.fileUploadProgressByUIInterface
}

export const getFilesByUIInterface = (): Record<string, File[]> => {
  const state = useChatUIStore.getState()
  return state.filesByUIInterface
}

export const getFileUploadProgressById = (
  uiInterface: string,
  progressId: string,
): FileUploadProgress | undefined => {
  const state = useChatUIStore.getState()
  const progressList = state.fileUploadProgressByUIInterface[uiInterface] || []
  return progressList.find((fp: FileUploadProgress) => fp.id === progressId)
}
