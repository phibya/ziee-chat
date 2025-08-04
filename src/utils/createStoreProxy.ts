import type { StoreApi, UseBoundStore } from 'zustand/index'
import { useShallow } from 'zustand/react/shallow'

type ExtractState<T> = T extends UseBoundStore<StoreApi<infer State>>
  ? State & {
      __state: State
    }
  : never

export const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): Readonly<ExtractState<T>> => {
  return new Proxy({} as Readonly<ExtractState<T>>, {
    get: (_, prop) => {
      if (prop === '__state') {
        return useStore.getState()
      }
      return useStore(
        useShallow((state: ExtractState<T>) => (state as any)[prop]),
      )
    },
  })
}
