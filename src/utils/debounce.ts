/**
 * Creates a debounced function that delays invoking func until after wait milliseconds
 * have elapsed since the last time the debounced function was invoked.
 *
 * Behaves like lodash.debounce with support for leading, trailing, and maxWait options.
 */

export interface DebounceOptions {
  /**
   * Specify invoking on the leading edge of the timeout.
   * @default false
   */
  leading?: boolean

  /**
   * The maximum time func is allowed to be delayed before it's invoked.
   */
  maxWait?: number

  /**
   * Specify invoking on the trailing edge of the timeout.
   * @default true
   */
  trailing?: boolean
}

export interface DebouncedFunc<T extends (...args: any[]) => any> {
  /**
   * Call the original function, but applying the debouncing rules.
   */
  (...args: Parameters<T>): ReturnType<T> | undefined

  /**
   * Throw away any pending invocation of the debounced function.
   */
  cancel(): void

  /**
   * Immediately call the original function with the most recent arguments passed to the debounced function.
   */
  flush(): ReturnType<T> | undefined

  /**
   * Check if there are any invocations pending.
   */
  pending(): boolean
}

/**
 * Creates a debounced function that delays invoking func until after wait milliseconds
 * have elapsed since the last time the debounced function was invoked.
 *
 * @param func The function to debounce.
 * @param wait The number of milliseconds to delay.
 * @param options The options object.
 * @returns Returns the new debounced function.
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number = 0,
  options: DebounceOptions = {},
): DebouncedFunc<T> {
  const { leading = false, maxWait, trailing = true } = options

  let lastArgs: Parameters<T> | undefined
  let lastThis: unknown
  let maxTimeoutId: ReturnType<typeof setTimeout> | undefined
  let result: ReturnType<T> | undefined
  let timerId: ReturnType<typeof setTimeout> | undefined
  let lastCallTime: number | undefined
  let lastInvokeTime = 0

  function invokeFunc(time: number): ReturnType<T> {
    const args = lastArgs!
    const thisArg = lastThis

    lastArgs = lastThis = undefined
    lastInvokeTime = time
    result = func.apply(thisArg, args) as ReturnType<T>
    return result as ReturnType<T>
  }

  function leadingEdge(time: number): ReturnType<T> | undefined {
    // Reset any `maxWait` timer.
    lastInvokeTime = time
    // Start the timer for the trailing edge.
    timerId = setTimeout(timerExpired, wait)
    // Invoke the leading edge.
    return leading ? invokeFunc(time) : result
  }

  function remainingWait(time: number): number {
    const timeSinceLastCall = time - lastCallTime!
    const timeSinceLastInvoke = time - lastInvokeTime
    const timeWaiting = wait - timeSinceLastCall

    return maxWait !== undefined
      ? Math.min(timeWaiting, maxWait - timeSinceLastInvoke)
      : timeWaiting
  }

  function shouldInvoke(time: number): boolean {
    const timeSinceLastCall = time - (lastCallTime || 0)
    const timeSinceLastInvoke = time - lastInvokeTime

    // Either this is the first call, activity has stopped and we're at the
    // trailing edge, the system time has gone backwards and we're treating
    // it as the trailing edge, or we've hit the `maxWait` limit.
    return (
      lastCallTime === undefined ||
      timeSinceLastCall >= wait ||
      timeSinceLastCall < 0 ||
      (maxWait !== undefined && timeSinceLastInvoke >= maxWait)
    )
  }

  function timerExpired(): ReturnType<T> | undefined {
    const time = Date.now()
    if (shouldInvoke(time)) {
      return trailingEdge(time)
    }
    // Restart the timer.
    timerId = setTimeout(timerExpired, remainingWait(time))
    return result
  }

  function trailingEdge(time: number): ReturnType<T> | undefined {
    timerId = undefined

    // Only invoke if we have `lastArgs` which means `func` has been
    // debounced at least once.
    if (trailing && lastArgs) {
      return invokeFunc(time)
    }
    lastArgs = lastThis = undefined
    return result
  }

  function cancel(): void {
    if (timerId !== undefined) {
      clearTimeout(timerId)
    }
    if (maxTimeoutId !== undefined) {
      clearTimeout(maxTimeoutId)
    }
    lastInvokeTime = 0
    lastArgs = lastCallTime = lastThis = timerId = maxTimeoutId = undefined
  }

  function flush(): ReturnType<T> | undefined {
    return timerId === undefined ? result : trailingEdge(Date.now())
  }

  function pending(): boolean {
    return timerId !== undefined
  }

  function debounced(
    this: unknown,
    ...args: Parameters<T>
  ): ReturnType<T> | undefined {
    const time = Date.now()
    const isInvoking = shouldInvoke(time)

    lastArgs = args
    lastThis = this
    lastCallTime = time

    if (isInvoking) {
      if (timerId === undefined) {
        return leadingEdge(lastCallTime)
      }
      if (maxWait !== undefined) {
        // Handle invocations in a tight loop.
        timerId = setTimeout(timerExpired, wait)
        return invokeFunc(lastCallTime)
      }
    }
    if (timerId === undefined) {
      timerId = setTimeout(timerExpired, wait)
    }
    return result
  }

  debounced.cancel = cancel
  debounced.flush = flush
  debounced.pending = pending

  return debounced
}

/**
 * Simple debounce function for basic use cases
 * @param func Function to debounce
 * @param delay Delay in milliseconds
 * @returns Debounced function
 */
export function simpleDebounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId)
    timeoutId = setTimeout(() => func(...args), delay)
  }
}
