import { Monitor, Moon, Sun } from 'lucide-react'
import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import SegmentedControl from '@/ui/atoms/SegmentedControl.tsx'

const OPTIONS: { icon: typeof Sun, label: string, value: 'dark' | 'light' | 'system' }[] = [
  { icon: Sun, label: 'Light', value: 'light' },
  { icon: Moon, label: 'Dark', value: 'dark' },
  { icon: Monitor, label: 'System', value: 'system' },
]

const AppearanceSettings = ({ onPreferenceChange, preference }: AppearanceSettingsArgs): React.JSX.Element => (
  <SegmentedControl name="theme-preference" onChange={onPreferenceChange} options={OPTIONS} value={preference} />
)

export default AppearanceSettings
