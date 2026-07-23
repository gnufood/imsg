import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import SettingsTemplate from '@/settings/templates/SettingsTemplate.tsx'
import type StatusPanelArgs from '@/settings/organisms/StatusPanel.types.ts'

interface SettingsProps {
  appearance: AppearanceSettingsArgs
  channelOverrides: ChannelOverridesArgs
  daemonControls: DaemonControlsArgs
  securityLevel: SecurityLevelArgs
  statusPanel: StatusPanelArgs
}

// Pure forwarder — the five grouped prop bags are built (and memoized) in `SettingsConnected`,
// The one place that actually owns the underlying hook state; this stays a plain pass-through so
// It never constructs a fresh object as a JSX prop itself (see internal/GUI_ATOMIC_DESIGN.md).
const Settings = ({ appearance, channelOverrides, daemonControls, securityLevel, statusPanel }: SettingsProps): React.JSX.Element => (
  <SettingsTemplate
    appearance={appearance}
    channelOverrides={channelOverrides}
    daemonControls={daemonControls}
    securityLevel={securityLevel}
    statusPanel={statusPanel}
  />
)

export default Settings
