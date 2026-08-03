import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import Settings from '@/settings/pages/Settings.tsx'
import useChannelOverrides from '@/settings/application/use-channel-overrides.ts'
import useConfig from '@/settings/application/use-config.ts'
import useDaemonActions from '@/settings/application/use-daemon-actions.ts'
import useDaemonStatus from '@/settings/application/use-daemon-status.ts'
import { useMemo } from 'react'
import useSecurityLevel from '@/settings/application/use-security-level.ts'

const SettingsConnected = ({ onPreferenceChange, preference }: AppearanceSettingsArgs): React.JSX.Element => {
  const { config, failed: configFailed, reload } = useConfig()
  const address = config?.device_address
  const { pollFailed: statusPollFailed, resumePolling, status } = useDaemonStatus(address)
  const daemonControls = useDaemonActions(address, reload)
  const channelOverrides = useChannelOverrides({
    address,
    mapChannel: config?.map_channel,
    onSaved: reload,
    pbapChannel: config?.pbap_channel,
  })
  const securityLevel = useSecurityLevel({ committedLevel: config?.security_level, onSaved: reload })

  const appearance = useMemo(() => ({ onPreferenceChange, preference }), [onPreferenceChange, preference])

  const statusPanel = useMemo(
    () => ({ address, configFailed, onResumeStatusPolling: resumePolling, onRetryConfig: reload, status, statusPollFailed }),
    [address, configFailed, resumePolling, reload, status, statusPollFailed],
  )

  return (
    <Settings appearance={appearance} channelOverrides={channelOverrides} daemonControls={daemonControls} securityLevel={securityLevel} statusPanel={statusPanel} />
  )
}

export default SettingsConnected
