import { useCallback, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import Button from '@/ui/atoms/Button.tsx'
import Checkbox from '@/ui/atoms/Checkbox.tsx'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import Text from '@/ui/atoms/Text.tsx'

interface ConnectionActionsArgs {
  onRestart: () => void
  onStop: () => void
  restartError: string | undefined
  restarting: boolean
  stopError: string | undefined
  stopping: boolean
}

const renderConnectionActions = ({ onRestart, onStop, restartError, restarting, stopError, stopping }: ConnectionActionsArgs): React.JSX.Element => (
  <>
    <div className="flex gap-2">
      <Button disabled={stopping} onClick={onStop}>
        Stop
      </Button>
      <Button disabled={restarting} onClick={onRestart}>
        Restart
      </Button>
    </div>
    {stopError !== undefined && (
      <Text size="xs" tone="muted">
        {stopError}
      </Text>
    )}
    {restartError !== undefined && (
      <Text size="xs" tone="muted">
        {restartError}
      </Text>
    )}
  </>
)

interface InstallActionsArgs {
  confirmUninstallOpen: boolean
  installError: string | undefined
  installing: boolean
  onCancelUninstall: () => void
  onConfirmUninstall: () => void
  onInstall: () => void
  onRequestUninstall: () => void
  onSystemChange: (system: boolean) => void
  system: boolean
  uninstallError: string | undefined
  uninstalling: boolean
}

const renderInstallActions = ({
  confirmUninstallOpen,
  installError,
  installing,
  onCancelUninstall,
  onConfirmUninstall,
  onInstall,
  onRequestUninstall,
  onSystemChange,
  system,
  uninstallError,
  uninstalling,
}: InstallActionsArgs): React.JSX.Element => (
  <>
    <Checkbox checked={system} label="Install as a system service" onChange={onSystemChange} />
    <div className="flex gap-2">
      <Button disabled={installing} onClick={onInstall}>
        Install
      </Button>
      <Button disabled={uninstalling} onClick={onRequestUninstall}>
        Uninstall
      </Button>
    </div>
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
    <AnimatePresence>
      {confirmUninstallOpen && (
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
  </>
)

interface UninstallConfirmState {
  confirmUninstallOpen: boolean
  onCancelUninstall: () => void
  onConfirmUninstall: () => void
  onRequestUninstall: () => void
}

// Pulled out of the component so `DaemonControls` itself stays under this repo's
// Max-lines-per-function limit — still organism-local UI state, just packaged as a local hook.
const useUninstallConfirm = (onUninstall: (system: boolean) => void, system: boolean): UninstallConfirmState => {
  const [confirmUninstallOpen, setConfirmUninstallOpen] = useState(false)

  const onRequestUninstall = useCallback(() => {
    setConfirmUninstallOpen(true)
  }, [])

  const onCancelUninstall = useCallback(() => {
    setConfirmUninstallOpen(false)
  }, [])

  const onConfirmUninstall = useCallback(() => {
    setConfirmUninstallOpen(false)
    onUninstall(system)
  }, [onUninstall, system])

  return { confirmUninstallOpen, onCancelUninstall, onConfirmUninstall, onRequestUninstall }
}

// `system` is one shared toggle for both install and uninstall — they're a matched `--system`
// Pair, not independently scoped controls.
const DaemonControls = ({
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
}: DaemonControlsArgs): React.JSX.Element => {
  const [system, setSystem] = useState(false)
  const { confirmUninstallOpen, onCancelUninstall, onConfirmUninstall, onRequestUninstall } = useUninstallConfirm(onUninstall, system)

  const handleInstall = useCallback(() => {
    onInstall(system)
  }, [onInstall, system])

  return (
    <div className="flex flex-col gap-4">
      {renderConnectionActions({ onRestart, onStop, restartError, restarting, stopError, stopping })}
      {renderInstallActions({
        confirmUninstallOpen,
        installError,
        installing,
        onCancelUninstall,
        onConfirmUninstall,
        onInstall: handleInstall,
        onRequestUninstall,
        onSystemChange: setSystem,
        system,
        uninstallError,
        uninstalling,
      })}
    </div>
  )
}

export default DaemonControls
