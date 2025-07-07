import { create } from 'zustand'

interface Thread {
  id: string
  title: string
  messages: Message[]
  createdAt: string
  updatedAt: string
  starred?: boolean
}

interface Message {
  id: string
  content: string
  role: 'user' | 'assistant' | 'system'
  timestamp: string
  threadId: string
}

interface Assistant {
  id: string
  name: string
  description: string
  model: string
  systemPrompt?: string
}

interface AppState {
  // UI State
  currentThreadId: string | null

  // Data
  threads: Thread[]
  messages: Message[]
  assistants: Assistant[]

  // Actions
  setCurrentThreadId: (threadId: string | null) => void

  // Thread actions
  createThread: (title: string) => string
  deleteThread: (threadId: string) => void
  updateThread: (threadId: string, updates: Partial<Thread>) => void
  starThread: (threadId: string) => void

  // Message actions
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void
  updateMessage: (messageId: string, updates: Partial<Message>) => void
  deleteMessage: (messageId: string) => void

  // Assistant actions
  createAssistant: (assistant: Omit<Assistant, 'id'>) => void
  updateAssistant: (assistantId: string, updates: Partial<Assistant>) => void
  deleteAssistant: (assistantId: string) => void
}

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  currentThreadId: null,
  threads: [],
  messages: [],
  assistants: [
    {
      id: 'default',
      name: 'Default Assistant',
      description: 'A helpful AI assistant',
      model: 'gpt-3.5-turbo',
    },
  ],

  // Actions
  setCurrentThreadId: threadId => set({ currentThreadId: threadId }),

  // Thread actions
  createThread: title => {
    const newThread: Thread = {
      id: crypto.randomUUID(),
      title,
      messages: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
    set(state => ({
      threads: [newThread, ...state.threads],
      currentThreadId: newThread.id,
    }))
    return newThread.id
  },

  deleteThread: threadId => {
    set(state => ({
      threads: state.threads.filter(t => t.id !== threadId),
      messages: state.messages.filter(m => m.threadId !== threadId),
      currentThreadId:
        state.currentThreadId === threadId ? null : state.currentThreadId,
    }))
  },

  updateThread: (threadId, updates) => {
    set(state => ({
      threads: state.threads.map(t =>
        t.id === threadId
          ? { ...t, ...updates, updatedAt: new Date().toISOString() }
          : t,
      ),
    }))
  },

  starThread: threadId => {
    set(state => ({
      threads: state.threads.map(t =>
        t.id === threadId
          ? { ...t, starred: !t.starred, updatedAt: new Date().toISOString() }
          : t,
      ),
    }))
  },

  // Message actions
  addMessage: message => {
    const newMessage: Message = {
      ...message,
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
    }
    set(state => ({
      messages: [...state.messages, newMessage],
    }))

    // Update thread's updatedAt
    const { updateThread } = get()
    updateThread(message.threadId, {})
  },

  updateMessage: (messageId, updates) => {
    set(state => ({
      messages: state.messages.map(m =>
        m.id === messageId ? { ...m, ...updates } : m,
      ),
    }))
  },

  deleteMessage: messageId => {
    set(state => ({
      messages: state.messages.filter(m => m.id !== messageId),
    }))
  },

  // Assistant actions
  createAssistant: assistant => {
    const newAssistant: Assistant = {
      ...assistant,
      id: crypto.randomUUID(),
    }
    set(state => ({
      assistants: [...state.assistants, newAssistant],
    }))
  },

  updateAssistant: (assistantId, updates) => {
    set(state => ({
      assistants: state.assistants.map(a =>
        a.id === assistantId ? { ...a, ...updates } : a,
      ),
    }))
  },

  deleteAssistant: assistantId => {
    set(state => ({
      assistants: state.assistants.filter(a => a.id !== assistantId),
    }))
  },
}))
