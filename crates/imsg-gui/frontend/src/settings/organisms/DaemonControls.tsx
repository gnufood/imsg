import { useCallback, useMemo, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import ServiceStatusCard from '@/ui/molecules/ServiceStatusCard.tsx'
import Text from '@/ui/atoms/Text.tsx'

type ServiceLevel = 'system' | 'user'

interface ServiceDescriptor {
  description: string
  level: ServiceLevel
  title: string
}

const SERVICES: ServiceDescriptor[] = [
  { description: 'Runs when you sign in — no administrator access.', level: 'user', title: 'User service' },
  { description: 'Runs for all users — administrator access required.', level: 'system', title: 'System service' },
]

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

interface ServiceRow extends ServiceDescriptor {
  handleToggle: () => void
  installed: boolean | undefined
}

// Stable per-service handlers, built once per `installedByLevel`/`onToggle` identity rather than
// As a fresh closure per render inside the JSX below (same reasoning as `SegmentedControl`'s own
// `buildSegments` — see `jsx-no-new-function-as-prop`).
const buildServiceRows = (
  installedByLevel: Record<ServiceLevel, boolean | undefined>,
  onToggle: (level: ServiceLevel, installed: boolean | undefined) => void,
): ServiceRow[] =>
  SERVICES.map((service) => ({
    description: service.description,
    handleToggle: () => {
      onToggle(service.level, installedByLevel[service.level])
    },
    installed: installedByLevel[service.level],
    level: service.level,
    title: service.title,
  }))

interface UseServiceRowsArgs {
  onInstall: (system: boolean) => void
  onRequestUninstall: (level: ServiceLevel) => void
  systemInstalled: boolean | undefined
  userInstalled: boolean | undefined
}

// Pulled out for the same max-lines-per-function reason as `useUninstallConfirm` below — also
// Where `onInstall`/`onRequestUninstall` collapse into the single per-service `onToggle` each row
// Needs, so `DaemonControls` itself never has to know which of the two a given row is mid-action.
const useServiceRows = ({ onInstall, onRequestUninstall, systemInstalled, userInstalled }: UseServiceRowsArgs): ServiceRow[] => {
  const installedByLevel = useMemo(() => ({ system: systemInstalled, user: userInstalled }), [systemInstalled, userInstalled])
  const onToggle = useCallback(
    (level: ServiceLevel, installed: boolean | undefined) => {
      if (installed === true) {
        onRequestUninstall(level)
      } else {
        onInstall(level === 'system')
      }
    },
    [onInstall, onRequestUninstall],
  )
  return useMemo(() => buildServiceRows(installedByLevel, onToggle), [installedByLevel, onToggle])
}

interface ServiceRowsArgs {
  pending: boolean
  rows: ServiceRow[]
}

const renderServiceRows = ({ pending, rows }: ServiceRowsArgs): React.JSX.Element => (
  <div className="flex flex-col gap-3">
    {rows.map((row) => (
      <ServiceStatusCard
        key={row.level}
        description={row.description}
        disabled={pending}
        installed={row.installed}
        onToggle={row.handleToggle}
        title={row.title}
      />
    ))}
  </div>
)

interface UninstallConfirmState {
  confirmLevel: ServiceLevel | undefined
  onCancelUninstall: () => void
  onConfirmUninstall: () => void
  onRequestUninstall: (level: ServiceLevel) => void
}

// Pulled out of the component so `DaemonControls` itself stays under this repo's
// Max-lines-per-function limit — still organism-local UI state, just packaged as a local hook.
// Level-scoped (unlike a single shared confirm flag) since either card can trigger it.
const useUninstallConfirm = (onUninstall: (system: boolean) => void): UninstallConfirmState => {
  const [confirmLevel, setConfirmLevel] = useState<ServiceLevel | undefined>()

  const onRequestUninstall = useCallback((level: ServiceLevel) => {
    setConfirmLevel(level)
  }, [])

  const onCancelUninstall = useCallback(() => {
    setConfirmLevel(undefined)
  }, [])

  const onConfirmUninstall = useCallback(() => {
    if (confirmLevel !== undefined) {
      onUninstall(confirmLevel === 'system')
    }
    setConfirmLevel(undefined)
  }, [confirmLevel, onUninstall])

  return { confirmLevel, onCancelUninstall, onConfirmUninstall, onRequestUninstall }
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
  const { confirmLevel, onCancelUninstall, onConfirmUninstall, onRequestUninstall } = useUninstallConfirm(onUninstall)
  const rows = useServiceRows({ onInstall, onRequestUninstall, systemInstalled, userInstalled })

  if (serviceStatusPollFailed) {
    return <ErrorState message="Couldn't check installed services." onRetry={onResumeServiceStatusPolling} />
  }

  return (
    <div className="flex flex-col gap-4">
      {renderServiceRows({ pending: installing || uninstalling, rows })}
      {renderActionErrors({ installError, uninstallError })}
      <AnimatePresence>
        {confirmLevel !== undefined && (
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
