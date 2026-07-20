import { Monitor, Moon, Sun } from 'lucide-react'
import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import SegmentedControl from '@/ui/atoms/SegmentedControl.tsx'

// `typeof Sun` (not a separate `LucideIcon` import) to avoid a duplicate-import from
// 'lucide-react' — same workaround as ErrorState/EmptyState.
const OPTIONS: { icon: typeof Sun, label: string, value: 'dark' | 'light' | 'system' }[] = [
  { icon: Sun, label: 'Light', value: 'light' },
  { icon: Moon, label: 'Dark', value: 'dark' },
  { icon: Monitor, label: 'System', value: 'system' },
]

// No loading/error/disabled state of its own — unlike `DaemonControls`/`ChannelOverridesForm`,
// There's no in-flight request here (see internal/GUI_ATOMIC_DESIGN.md); it's a synchronous
// Preference the application layer persists.
const AppearanceSettings = ({ onPreferenceChange, preference }: AppearanceSettingsArgs): React.JSX.Element => (
  <SegmentedControl name="theme-preference" onChange={onPreferenceChange} options={OPTIONS} value={preference} />
)

export default AppearanceSettings
