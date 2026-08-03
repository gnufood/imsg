import Settings from '@/settings/pages/Settings.tsx'
import useChannelOverrides from '@/settings/application/use-channel-overrides.ts'
import useConfig from '@/settings/application/use-config.ts'
import useDaemonActions from '@/settings/application/use-daemon-actions.ts'
import useDaemonStatus from '@/settings/application/use-daemon-status.ts'
import { useMemo } from 'react'

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between the
// Settings hooks' real backend calls and `Settings`'s presentational page. Also the one place
// That groups their outputs into the per-organism prop bags `Settings`/`SettingsTemplate` just
// Forward, memoized here since it's the actual owner of the underlying state.
const SettingsConnected = (): React.JSX.Element => {
  const { config, failed: configFailed, reload } = useConfig()
  const address = config?.device_address
  const { pollFailed: statusPollFailed, resumePolling, status } = useDaemonStatus(address)
  const daemonControls = useDaemonActions(address, reload)
  const { error: channelError, mapDraft, onMapDraftChange, onPbapDraftChange, pbapDraft, save, saving: savingChannels } = useChannelOverrides(
    config?.map_channel,
    config?.pbap_channel,
    reload,
  )

  const statusPanel = useMemo(
    () => ({ address, configFailed, onResumeStatusPolling: resumePolling, onRetryConfig: reload, status, statusPollFailed }),
    [address, configFailed, resumePolling, reload, status, statusPollFailed],
  )

  const channelOverrides = useMemo(
    () => ({ error: channelError, mapDraft, onMapDraftChange, onPbapDraftChange, onSave: save, pbapDraft, saving: savingChannels }),
    [channelError, mapDraft, onMapDraftChange, onPbapDraftChange, save, pbapDraft, savingChannels],
  )

  return <Settings channelOverrides={channelOverrides} daemonControls={daemonControls} statusPanel={statusPanel} />
}

export default SettingsConnected
