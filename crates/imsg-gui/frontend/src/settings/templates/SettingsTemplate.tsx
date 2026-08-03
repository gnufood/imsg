import AppearanceSettings from '@/settings/organisms/AppearanceSettings.tsx'
import type AppearanceSettingsArgs from '@/settings/organisms/AppearanceSettings.types.ts'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import ChannelOverridesForm from '@/settings/organisms/ChannelOverridesForm.tsx'
import DaemonControls from '@/settings/organisms/DaemonControls.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import SecurityLevelForm from '@/settings/organisms/SecurityLevelForm.tsx'
import StatusPanel from '@/settings/organisms/StatusPanel.tsx'
import type StatusPanelArgs from '@/settings/organisms/StatusPanel.types.ts'
import Text from '@/ui/atoms/Text.tsx'

type SecurityLevelArgs = React.ComponentProps<typeof SecurityLevelForm>

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
  onResumeServiceStatusPolling,
  onStop,
  onUninstall,
  restartError,
  restarting,
  serviceStatusPollFailed,
  stopError,
  stopping,
  systemInstalled,
  uninstallError,
  uninstalling,
  userInstalled,
}: DaemonControlsArgs): React.JSX.Element => (
  <DaemonControls
    installError={installError}
    installing={installing}
    onInstall={onInstall}
    onRestart={onRestart}
    onResumeServiceStatusPolling={onResumeServiceStatusPolling}
    onStop={onStop}
    onUninstall={onUninstall}
    restartError={restartError}
    restarting={restarting}
    serviceStatusPollFailed={serviceStatusPollFailed}
    stopError={stopError}
    stopping={stopping}
    systemInstalled={systemInstalled}
    uninstallError={uninstallError}
    uninstalling={uninstalling}
    userInstalled={userInstalled}
  />
)

const renderAppearance = ({ onPreferenceChange, preference }: AppearanceSettingsArgs): React.JSX.Element => (
  <AppearanceSettings onPreferenceChange={onPreferenceChange} preference={preference} />
)

const renderSecurityLevel = ({
  committedLevel,
  draft,
  onCancel,
  onDraftChange,
  onSave,
  saveError,
  saving,
}: SecurityLevelArgs): React.JSX.Element => (
  <SecurityLevelForm
    committedLevel={committedLevel}
    draft={draft}
    onCancel={onCancel}
    onDraftChange={onDraftChange}
    onSave={onSave}
    saveError={saveError}
    saving={saving}
  />
)

const renderChannelOverrides = ({
  detectError,
  detecting,
  editorOpen,
  mapChannel,
  mapDraft,
  onCancel,
  onDetect,
  onMapDraftChange,
  onPbapDraftChange,
  onRequestEdit,
  onSave,
  pbapChannel,
  pbapDraft,
  saveError,
  saving,
}: ChannelOverridesArgs): React.JSX.Element => (
  <ChannelOverridesForm
    detectError={detectError}
    detecting={detecting}
    editorOpen={editorOpen}
    mapChannel={mapChannel}
    mapDraft={mapDraft}
    onCancel={onCancel}
    onDetect={onDetect}
    onMapDraftChange={onMapDraftChange}
    onPbapDraftChange={onPbapDraftChange}
    onRequestEdit={onRequestEdit}
    onSave={onSave}
    pbapChannel={pbapChannel}
    pbapDraft={pbapDraft}
    saveError={saveError}
    saving={saving}
  />
)

interface SettingsTemplateProps {
  appearance: AppearanceSettingsArgs
  channelOverrides: ChannelOverridesArgs
  daemonControls: DaemonControlsArgs
  securityLevel: SecurityLevelArgs
  statusPanel: StatusPanelArgs
}

const SettingsTemplate = ({ appearance, channelOverrides, daemonControls, securityLevel, statusPanel }: SettingsTemplateProps): React.JSX.Element => (
  <div className="h-full overflow-y-auto">
    <div className="mx-auto flex w-full max-w-3xl gap-x-10 p-6">
      <div className="flex flex-1 flex-col gap-8">
        <section className="flex flex-col gap-3">
          <Text as="h2" tone="accent">Device</Text>
          {renderStatusPanel(statusPanel)}
        </section>
        <section className="flex flex-col gap-3">
          <Text as="h2" tone="accent">Daemon</Text>
          {renderDaemonControls(daemonControls)}
        </section>
      </div>
      <div className="flex flex-1 flex-col gap-8">
        <section className="flex flex-col gap-3">
          <Text as="h2" tone="accent">Channels</Text>
          {renderChannelOverrides(channelOverrides)}
        </section>
        <section className="flex flex-col gap-3">
          <Text as="h2" tone="accent">Security</Text>
          {renderSecurityLevel(securityLevel)}
        </section>
        <section className="flex flex-col gap-3">
          <Text as="h2" tone="accent">Appearance</Text>
          {renderAppearance(appearance)}
        </section>
      </div>
    </div>
  </div>
)

export default SettingsTemplate
