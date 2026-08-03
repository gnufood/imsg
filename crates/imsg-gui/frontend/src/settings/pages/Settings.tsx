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
