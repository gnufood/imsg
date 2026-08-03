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
  } catch {}
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
    } catch {}
  }, [])

  return { preference: storedPreference, setPreference }
}

export default useThemePreference
