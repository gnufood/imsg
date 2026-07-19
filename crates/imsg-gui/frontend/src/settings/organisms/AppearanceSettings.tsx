import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import SegmentedControl from '@/ui/atoms/SegmentedControl.tsx'

const OPTIONS: { label: string, value: 'dark' | 'light' | 'system' }[] = [
  { label: 'Light', value: 'light' },
  { label: 'Dark', value: 'dark' },
  { label: 'System', value: 'system' },
]

// No loading/error/disabled state of its own — unlike `DaemonControls`/`ChannelOverridesForm`,
// There's no in-flight request here (see internal/GUI_ATOMIC_DESIGN.md); it's a synchronous
// Preference the application layer persists.
const AppearanceSettings = ({ onPreferenceChange, preference }: AppearanceSettingsArgs): React.JSX.Element => (
  <SegmentedControl name="theme-preference" onChange={onPreferenceChange} options={OPTIONS} value={preference} />
)

export default AppearanceSettings
