import { create } from 'zustand/index'

export const usePathHistoryStore = create<{
  previousSettingPagePath: string
}>(() => ({
  previousSettingPagePath: '/settings/general',
}))

export const setPreviousSettingPagePath = (path: string) => {
  usePathHistoryStore.setState({ previousSettingPagePath: path })
}
