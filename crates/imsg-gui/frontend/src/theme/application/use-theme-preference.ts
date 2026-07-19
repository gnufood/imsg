import { useCallback, useEffect, useState } from 'react'
import type UseThemePreferenceResult from '@/theme/application/use-theme-preference.types.ts'

type ThemePreference = UseThemePreferenceResult['preference']
type ResolvedTheme = 'dark' | 'light'

const STORAGE_KEY = 'imsg:theme-preference'
const SYSTEM_DARK_QUERY = '(prefers-color-scheme: dark)'

const isThemePreference = (value: string | null): value is ThemePreference =>
  value === 'dark' || value === 'light' || value === 'system'

const readStoredPreference = (): ThemePreference => {
  try {
    const stored = globalThis.localStorage.getItem(STORAGE_KEY)
    if (isThemePreference(stored)) {
      return stored
    }
  } catch {
    // `localStorage` can throw in restricted contexts (private mode, disabled storage) —
    // Fall back to 'system' rather than crash the app over a preference read.
  }
  return 'system'
}

const resolveSystemTheme = (): ResolvedTheme => {
  if (globalThis.matchMedia(SYSTEM_DARK_QUERY).matches) {
    return 'dark'
  }
  return 'light'
}

const resolveTheme = (preference: ThemePreference, systemTheme: ResolvedTheme): ResolvedTheme => {
  if (preference === 'system') {
    return systemTheme
  }
  return preference
}

// Application boundary for theming (see internal/GUI_ATOMIC_DESIGN.md) — the only file allowed
// To touch `localStorage`/`matchMedia`/`document.documentElement` directly. Only ever
// Instantiated once, by `AppReady` (post-gate — see that file), so its `data-theme` effect never
// Runs during Gate/Splash/DeviceSetup.
const useThemePreference = (): UseThemePreferenceResult => {
  const [storedPreference, setStoredPreference] = useState<ThemePreference>(readStoredPreference)
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(resolveSystemTheme)

  useEffect(() => {
    const query = globalThis.matchMedia(SYSTEM_DARK_QUERY)
    const handleChange = (event: MediaQueryListEvent): void => {
      let next: ResolvedTheme = 'light'
      if (event.matches) {
        next = 'dark'
      }
      setSystemTheme(next)
    }
    query.addEventListener('change', handleChange)
    return () => {
      query.removeEventListener('change', handleChange)
    }
  }, [])

  const resolvedTheme = resolveTheme(storedPreference, systemTheme)

  useEffect(() => {
    document.documentElement.dataset['theme'] = resolvedTheme
  }, [resolvedTheme])

  const setPreference = useCallback((next: ThemePreference) => {
    setStoredPreference(next)
    try {
      globalThis.localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // Same rationale as the read path — a failed write just means the choice won't persist
      // Across relaunches this session, not a reason to throw.
    }
  }, [])

  return { preference: storedPreference, setPreference }
}

export default useThemePreference
