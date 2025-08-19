import { create } from 'zustand/index'

export const usePathHistoryStore = create<{
  previousSettingPagePath: string
  previousConversationListPagePath: string
}>(() => ({
  previousSettingPagePath: '/settings/general',
  previousConversationListPagePath: '/conversations',
}))

export const setPreviousSettingPagePath = (path: string) => {
  usePathHistoryStore.setState({ previousSettingPagePath: path })
}

export const setPreviousConversationListPagePath = (path: string) => {
  usePathHistoryStore.setState({ previousConversationListPagePath: path })
}
