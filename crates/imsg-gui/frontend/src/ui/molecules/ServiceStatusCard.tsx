import Button from '@/ui/atoms/Button.tsx'
import StatusDot from '@/ui/atoms/StatusDot.tsx'
import Text from '@/ui/atoms/Text.tsx'

interface ServiceStatusCardProps {
  description: string
  disabled: boolean
  installed: boolean | undefined
  onToggle: () => void
  title: string
}

// One row of `DaemonControls`: a service level's identity (title/description), its
// Installed/not-installed state, and the single action that flips it. `installed === undefined`
// Means "not yet known" (first status fetch still pending), not "not installed" — the action
// Button stays disabled rather than guessing which action to offer.
const ServiceStatusCard = ({ description, disabled, installed, onToggle, title }: ServiceStatusCardProps): React.JSX.Element => {
  let statusLabel = 'Checking…'
  let statusTone: 'active' | 'idle' | 'pending' = 'pending'
  let actionLabel = 'Install'
  if (installed === true) {
    statusLabel = 'Installed'
    statusTone = 'active'
    actionLabel = 'Uninstall'
  } else if (installed === false) {
    statusLabel = 'Not installed'
    statusTone = 'idle'
  }

  return (
    <div className="flex items-center gap-3 rounded-lg border border-line px-3 py-2">
      <div className="flex flex-1 flex-col gap-0.5">
        <Text>{title}</Text>
        <Text size="xs" tone="muted">
          {description}
        </Text>
        <div className="flex items-center gap-1.5">
          <StatusDot tone={statusTone} />
          <Text size="xs" tone="muted">
            {statusLabel}
          </Text>
        </div>
      </div>
      <Button disabled={disabled || installed === undefined} onClick={onToggle}>
        {actionLabel}
      </Button>
    </div>
  )
}

export default ServiceStatusCard
