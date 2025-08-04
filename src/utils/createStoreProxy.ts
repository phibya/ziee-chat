import type { StoreApi, UseBoundStore } from 'zustand/index'
import { useShallow } from 'zustand/react/shallow'

type RemoveVoid<T> = T extends void ? never : T

type ExtractZustandState<T> = T extends UseBoundStore<infer Store>
  ? Store extends StoreApi<infer State>
    ? RemoveVoid<State> & { __state: RemoveVoid<State> }
    : Store extends { getState(): infer State }
      ? State extends void | infer S
        ? S extends void
          ? never
          : S
        : RemoveVoid<State> & { __state: RemoveVoid<State> }
      : never
  : never

export const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): Readonly<ExtractZustandState<T>> => {
  return new Proxy({} as Readonly<ExtractZustandState<T>>, {
    get: (_, prop) => {
      if (prop === '__state') {
        return useStore.getState()
      }
      return useStore(
        useShallow((state: ExtractZustandState<T>) => (state as any)[prop]),
      )
    },
  })
}
