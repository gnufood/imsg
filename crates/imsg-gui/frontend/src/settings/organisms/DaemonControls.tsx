import { useCallback, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import ServiceStatusCard from '@/ui/molecules/ServiceStatusCard.tsx'
import Text from '@/ui/atoms/Text.tsx'

const USER_SERVICE_DESCRIPTION = 'Runs in the user session at login. Unprivileged.'
const USER_SERVICE_TITLE = 'User service'

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

const renderServiceRows = ({ onToggleUser, pending, systemInstalled, userInstalled }: ServiceRowsArgs): React.JSX.Element => {
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
