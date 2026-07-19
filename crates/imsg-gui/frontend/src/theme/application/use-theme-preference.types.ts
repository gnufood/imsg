export default interface UseThemePreferenceResult {
  preference: 'dark' | 'light' | 'system'
  setPreference: (preference: 'dark' | 'light' | 'system') => void
}
