import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import ChannelOverridesForm from '@/settings/organisms/ChannelOverridesForm.tsx'
import DaemonControls from '@/settings/organisms/DaemonControls.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import StatusPanel from '@/settings/organisms/StatusPanel.tsx'
import type StatusPanelArgs from '@/settings/organisms/StatusPanel.types.ts'
import Text from '@/ui/atoms/Text.tsx'

// Each organism already handles its own loading/error/pending states internally (see
// `StatusPanel`/`DaemonControls`/`ChannelOverridesForm`), so unlike `MessagesTemplate` there's no
// Whole-screen gate here. Props are grouped 1:1 with the three organisms — each already has its
// Own `*.types.ts` (see internal/GUI_ATOMIC_DESIGN.md's convention) — and each gets its own
// Render helper purely to stay within this repo's max-lines-per-function limit.
const renderStatusPanel = ({
  address,
  configFailed,
  onResumeStatusPolling,
  onRetryConfig,
  status,
  statusPollFailed,
}: StatusPanelArgs): React.JSX.Element => (
  <StatusPanel
    address={address}
    configFailed={configFailed}
    onResumeStatusPolling={onResumeStatusPolling}
    onRetryConfig={onRetryConfig}
    status={status}
    statusPollFailed={statusPollFailed}
  />
)

const renderDaemonControls = ({
  installError,
  installing,
  onInstall,
  onRestart,
  onStop,
  onUninstall,
  restartError,
  restarting,
  stopError,
  stopping,
  uninstallError,
  uninstalling,
}: DaemonControlsArgs): React.JSX.Element => (
  <DaemonControls
    installError={installError}
    installing={installing}
    onInstall={onInstall}
    onRestart={onRestart}
    onStop={onStop}
    onUninstall={onUninstall}
    restartError={restartError}
    restarting={restarting}
    stopError={stopError}
    stopping={stopping}
    uninstallError={uninstallError}
    uninstalling={uninstalling}
  />
)

const renderChannelOverrides = ({
  error,
  mapDraft,
  onMapDraftChange,
  onPbapDraftChange,
  onSave,
  pbapDraft,
  saving,
}: ChannelOverridesArgs): React.JSX.Element => (
  <ChannelOverridesForm
    error={error}
    mapDraft={mapDraft}
    onMapDraftChange={onMapDraftChange}
    onPbapDraftChange={onPbapDraftChange}
    onSave={onSave}
    pbapDraft={pbapDraft}
    saving={saving}
  />
)

interface SettingsTemplateProps {
  channelOverrides: ChannelOverridesArgs
  daemonControls: DaemonControlsArgs
  statusPanel: StatusPanelArgs
}

const SettingsTemplate = ({ channelOverrides, daemonControls, statusPanel }: SettingsTemplateProps): React.JSX.Element => (
  <div className="mx-auto flex w-full max-w-lg flex-col gap-8 p-6">
    <section className="flex flex-col gap-3">
      <Text as="h2" tone="muted">Device</Text>
      {renderStatusPanel(statusPanel)}
    </section>
    <section className="flex flex-col gap-3">
      <Text as="h2" tone="muted">Daemon</Text>
      {renderDaemonControls(daemonControls)}
    </section>
    <section className="flex flex-col gap-3">
      <Text as="h2" tone="muted">Channels</Text>
      {renderChannelOverrides(channelOverrides)}
    </section>
  </div>
)

export default SettingsTemplate
