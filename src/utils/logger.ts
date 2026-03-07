const debugEnabled = import.meta.env.DEV || import.meta.env.VITE_DEBUG_LOGS === 'true'

export function devLog(...args: unknown[]) {
  if (debugEnabled) {
    console.log(...args)
  }
}

export function devWarn(...args: unknown[]) {
  if (debugEnabled) {
    console.warn(...args)
  }
}

export function isDebugLoggingEnabled() {
  return debugEnabled
}
