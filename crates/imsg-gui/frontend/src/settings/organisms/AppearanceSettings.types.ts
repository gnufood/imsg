export default interface AppearanceSettingsArgs {
  onPreferenceChange: (preference: 'dark' | 'light' | 'system') => void
  preference: 'dark' | 'light' | 'system'
}
