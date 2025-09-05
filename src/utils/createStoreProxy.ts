import type { StoreApi, UseBoundStore } from 'zustand/index'
import { useShallow } from 'zustand/react/shallow'

type RemoveVoid<T> = T extends void ? never : T

type ExtractZustandState<T> = T extends UseBoundStore<infer Store>
  ? Store extends StoreApi<infer State>
    ? RemoveVoid<State> & {
        __state: RemoveVoid<State>
        __setState: StoreApi<State>['setState']
      }
    : Store extends { getState(): infer State }
      ? State extends void | infer S
        ? S extends void
          ? never
          : S
        : RemoveVoid<State> & {
            __state: RemoveVoid<State>
            __setState: any
          }
      : never
  : never

export const createStoreProxy = <T extends UseBoundStore<StoreApi<any>>>(
  useStore: T,
): Readonly<ExtractZustandState<T>> => {
  const propInitCheck = new Map<string | symbol, boolean>()
  return new Proxy({} as Readonly<ExtractZustandState<T>>, {
    get: (_, prop) => {
      if (prop === '__state') {
        return useStore.getState()
      }
      if (prop === '__setState') {
        return useStore.setState.bind(useStore)
      }

      const isInit = propInitCheck.get(prop) || false
      if (!isInit) {
        let state = useStore.getState()
        if (state.__init__ && typeof state.__init__[prop] === 'function') {
          state.__init__[prop]()
        }
        propInitCheck.set(prop, true)
      }

      return useStore(
        useShallow((state: ExtractZustandState<T>) => (state as any)[prop]),
      )
    },
  })
}
