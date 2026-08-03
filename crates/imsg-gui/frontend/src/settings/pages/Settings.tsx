import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import SettingsTemplate from '@/settings/templates/SettingsTemplate.tsx'
import type StatusPanelArgs from '@/settings/organisms/StatusPanel.types.ts'

interface SettingsProps {
  channelOverrides: ChannelOverridesArgs
  daemonControls: DaemonControlsArgs
  statusPanel: StatusPanelArgs
}

// Pure forwarder — the three grouped prop bags are built (and memoized) in `SettingsConnected`,
// The one place that actually owns the underlying hook state; this stays a plain pass-through so
// It never constructs a fresh object as a JSX prop itself (see internal/GUI_ATOMIC_DESIGN.md).
const Settings = ({ channelOverrides, daemonControls, statusPanel }: SettingsProps): React.JSX.Element => (
  <SettingsTemplate channelOverrides={channelOverrides} daemonControls={daemonControls} statusPanel={statusPanel} />
)

export default Settings
