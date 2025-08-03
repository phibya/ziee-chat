import { create } from 'zustand'
import type { File, FileUploadProgress } from '../../types'
import { ApiClient } from '../../api/client.ts'

export type ChatUIInterfaceID = 'new-chat' | string

interface ChatUIState {
  // Message editing state
  editingMessageId: string | null
  editingMessageContent: string
  showMessageToolBox: { [messageId: string]: boolean }

  // Chat input state
  inputDisabled: boolean
  inputPlaceholder: string
  //UIInterface can be new-chat, {projectId}, or {conversationId} or {messageId} (for editing)
  fileUploadProgressByUIInterface: Record<string, FileUploadProgress[]>
  filesByUIInterface: Record<string, File[]>
  newFilesByUIInterface: Record<string, File[]>
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
  newFilesByUIInterface: {},
}))

// Actions
export const startEditingMessage = ({
  messageId,
  content,
  files = [],
}: {
  messageId: string
  content: string
  files?: File[]
}) => {
  useChatUIStore.setState({
    editingMessageId: messageId,
    editingMessageContent: content,
    showMessageToolBox: {
      ...useChatUIStore.getState().showMessageToolBox,
      [messageId]: false, // Hide toolbox when editing
    },
    filesByUIInterface: {
      ...useChatUIStore.getState().filesByUIInterface,
      [messageId]: files, // Set files for this message
    },
    fileUploadProgressByUIInterface: {
      ...useChatUIStore.getState().fileUploadProgressByUIInterface,
      [messageId]: [], // Initialize upload progress for this message
    },
  })
}

export const stopEditingMessage = () => {
  let editingMessageId = useChatUIStore.getState().editingMessageId
  if (!editingMessageId) return
  let showMessageToolBox = useChatUIStore.getState().showMessageToolBox
  delete showMessageToolBox[editingMessageId]

  let filesByUIInterface = useChatUIStore.getState().filesByUIInterface
  delete filesByUIInterface[editingMessageId]

  let newFilesByUIInterface = useChatUIStore.getState().newFilesByUIInterface
  delete newFilesByUIInterface[editingMessageId]

  let fileUploadProgressByUIInterface =
    useChatUIStore.getState().fileUploadProgressByUIInterface
  delete fileUploadProgressByUIInterface[editingMessageId]

  useChatUIStore.setState({
    editingMessageId: null,
    editingMessageContent: '',
    showMessageToolBox: {
      ...showMessageToolBox,
    },
    filesByUIInterface: {
      ...filesByUIInterface,
    },
    newFilesByUIInterface: {
      ...newFilesByUIInterface,
    },
    fileUploadProgressByUIInterface: {
      ...fileUploadProgressByUIInterface,
    },
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

export const resetChatUI = (uiInterfaceId?: string) => {
  if (uiInterfaceId) {
    // Reset specific UI interface
    useChatUIStore.setState(state => ({
      fileUploadProgressByUIInterface: {
        ...state.fileUploadProgressByUIInterface,
        [uiInterfaceId]: [],
      },
      filesByUIInterface: {
        ...state.filesByUIInterface,
        [uiInterfaceId]: [],
      },
      newFilesByUIInterface: {
        ...state.newFilesByUIInterface,
        [uiInterfaceId]: [],
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
      newFilesByUIInterface: {},
    })
  }
}

// File upload actions
export const uploadFilesToChat = async (
  uiInterfaceId: string,
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
        [uiInterfaceId]: [
          ...(state.fileUploadProgressByUIInterface[uiInterfaceId] || []),
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
          [uiInterfaceId]: (
            state.fileUploadProgressByUIInterface[uiInterfaceId] || []
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
                  [uiInterfaceId]: (
                    state.fileUploadProgressByUIInterface[uiInterfaceId] || []
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
                  [uiInterfaceId]: (
                    state.fileUploadProgressByUIInterface[uiInterfaceId] || []
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
                  [uiInterfaceId]: (
                    state.fileUploadProgressByUIInterface[uiInterfaceId] || []
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

        useChatUIStore.setState(state => ({
          newFilesByUIInterface: {
            ...state.newFilesByUIInterface,
            [uiInterfaceId]: [
              ...(state.newFilesByUIInterface[uiInterfaceId] || []),
              response.file,
            ],
          },
        }))
      } catch (fileError) {
        // Mark this file as failed
        useChatUIStore.setState(state => ({
          fileUploadProgressByUIInterface: {
            ...state.fileUploadProgressByUIInterface,
            [uiInterfaceId]: (
              state.fileUploadProgressByUIInterface[uiInterfaceId] || []
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

    // Note: Completed files are automatically removed from progress on completion
    // Only failed files remain in progress for user to see errors

    return uploadedFiles
  } catch (error) {
    useChatUIStore.setState(state => ({
      fileUploadProgressByUIInterface: {
        ...state.fileUploadProgressByUIInterface,
        [uiInterfaceId]: [],
      },
    }))
    throw error
  }
}

export const removeFileFromChat = (uiInterfaceId: string, fileId: string) => {
  useChatUIStore.setState(state => ({
    filesByUIInterface: {
      ...state.filesByUIInterface,
      [uiInterfaceId]: (state.filesByUIInterface[uiInterfaceId] || []).filter(
        (file: File) => file.id !== fileId,
      ),
    },
    newFilesByUIInterface: {
      ...state.newFilesByUIInterface,
      [uiInterfaceId]: (
        state.newFilesByUIInterface[uiInterfaceId] || []
      ).filter((file: File) => file.id !== fileId),
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
