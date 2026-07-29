import { useCallback, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import ServiceStatusCard from '@/ui/molecules/ServiceStatusCard.tsx'
import Text from '@/ui/atoms/Text.tsx'

const USER_SERVICE_DESCRIPTION = 'Runs in the user session at login. Unprivileged.'
const USER_SERVICE_TITLE = 'User service'

// The system row is status-only. Reading its state is unprivileged (`systemctl status` needs no
// Root), but installing it writes to root-owned directories, and neither Tauri nor
// `service-manager` exposes a way to elevate — so the action would always fail. It stays a CLI
// Operation (`sudo imsg daemon install --system`). Showing the state still matters: a
// CLI-installed system daemon would otherwise be invisible here while the user installs a second
// One at user level, and the two would contend for the same RFCOMM channel and socket.
const SYSTEM_SERVICE_DESCRIPTION = 'Runs at boot, independent of login. Install from a terminal — requires root.'
const SYSTEM_SERVICE_TITLE = 'System service'

interface ActionErrorsArgs {
  installError: string | undefined
  uninstallError: string | undefined
}

const renderActionErrors = ({ installError, uninstallError }: ActionErrorsArgs): React.JSX.Element => (
  <>
    {installError !== undefined && (
      <Text size="xs" tone="muted">
        {installError}
      </Text>
    )}
    {uninstallError !== undefined && (
      <Text size="xs" tone="muted">
        {uninstallError}
      </Text>
    )}
  </>
)

const CONTENTION_NOTE =
  'A system service is already installed. Both levels would contend for the same RFCOMM channel and IPC socket, so remove it first with `sudo imsg daemon uninstall --system`.'

interface ServiceRowsArgs {
  onToggleUser: () => void
  pending: boolean
  systemInstalled: boolean | undefined
  userInstalled: boolean | undefined
}

// Extracted for the same max-lines-per-function reason as `useUninstallConfirm` below.
const renderServiceRows = ({ onToggleUser, pending, systemInstalled, userInstalled }: ServiceRowsArgs): React.JSX.Element => {
  // Gates installing only. Uninstall stays available so an already-installed user service can
  // Always be removed — blocking that would trap the user in the contending state.
  const blockedBySystem = systemInstalled === true && userInstalled !== true
  return (
    <div className="flex flex-col gap-3">
      <ServiceStatusCard
        description={USER_SERVICE_DESCRIPTION}
        disabled={pending || blockedBySystem}
        installed={userInstalled}
        onToggle={onToggleUser}
        title={USER_SERVICE_TITLE}
      />
      <ServiceStatusCard
        description={SYSTEM_SERVICE_DESCRIPTION}
        disabled={pending}
        installed={systemInstalled}
        onToggle={undefined}
        title={SYSTEM_SERVICE_TITLE}
      />
      {blockedBySystem && (
        <Text size="xs" tone="muted">
          {CONTENTION_NOTE}
        </Text>
      )}
    </div>
  )
}

interface UninstallConfirmState {
  confirming: boolean
  onCancelUninstall: () => void
  onConfirmUninstall: () => void
  onRequestUninstall: () => void
}

// Pulled out of the component so `DaemonControls` itself stays under this repo's
// Max-lines-per-function limit — still organism-local UI state, just packaged as a local hook.
const useUninstallConfirm = (onUninstall: () => void): UninstallConfirmState => {
  const [confirming, setConfirming] = useState(false)

  const onRequestUninstall = useCallback(() => {
    setConfirming(true)
  }, [])

  const onCancelUninstall = useCallback(() => {
    setConfirming(false)
  }, [])

  const onConfirmUninstall = useCallback(() => {
    onUninstall()
    setConfirming(false)
  }, [onUninstall])

  return { confirming, onCancelUninstall, onConfirmUninstall, onRequestUninstall }
}

// Stop/Restart are intentionally not rendered here (unlike Install/Uninstall/status, which are
// Now wired to real `imsg-service` calls). `run_headless`'s `tracing::` output is currently
// Silently dropped — no subscriber installed (see internal/GUI.md's "Logging/verbosity setup",
// Decided but not built). Offering controls that can leave the daemon stopped with no way to see
// Whether it's alive again, or why it isn't, is worse than not offering them.
// `DaemonControlsArgs`/`useDaemonActions` still carry the full stop/restart surface unchanged —
// This is a one-line revert once that lands.
const DaemonControls = ({
  installError,
  installing,
  onInstall,
  onResumeServiceStatusPolling,
  onUninstall,
  serviceStatusPollFailed,
  systemInstalled,
  uninstallError,
  uninstalling,
  userInstalled,
}: DaemonControlsArgs): React.JSX.Element => {
  const { confirming, onCancelUninstall, onConfirmUninstall, onRequestUninstall } = useUninstallConfirm(onUninstall)
  const onToggle = useCallback(() => {
    if (userInstalled === true) {
      onRequestUninstall()
    } else {
      onInstall()
    }
  }, [onInstall, onRequestUninstall, userInstalled])

  if (serviceStatusPollFailed) {
    return <ErrorState message="Couldn't check installed services." onRetry={onResumeServiceStatusPolling} />
  }

  return (
    <div className="flex flex-col gap-4">
      {renderServiceRows({ onToggleUser: onToggle, pending: installing || uninstalling, systemInstalled, userInstalled })}
      {renderActionErrors({ installError, uninstallError })}
      <AnimatePresence>
        {confirming && (
          <ConfirmDialog
            confirmLabel="Uninstall"
            error={undefined}
            message="Uninstall the daemon service? You can reinstall it later from this screen."
            onCancel={onCancelUninstall}
            onConfirm={onConfirmUninstall}
            pending={uninstalling}
            title="Uninstall daemon service?"
          />
        )}
      </AnimatePresence>
    </div>
  )
}

export default DaemonControls
