import Button from '@/ui/atoms/Button.tsx'
import StatusDot from '@/ui/atoms/StatusDot.tsx'
import Text from '@/ui/atoms/Text.tsx'

interface ServiceStatusCardProps {
  description: string
  disabled: boolean
  installed: boolean | undefined
  onToggle: (() => void) | undefined
  title: string
}

// One row of `DaemonControls`: a service level's identity (title/description), its
// Installed/not-installed state, and the single action that flips it. `installed === undefined`
// Means "not yet known" (first status fetch still pending), not "not installed" — the action
// Button stays disabled rather than guessing which action to offer.
//
// `onToggle === undefined` renders the row read-only: status is still reported, but no action is
// Offered. Used for a service level this process can observe but can't modify.
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
    <div className="flex items-center gap-3 py-1">
      <div className="flex flex-1 flex-col gap-0.5">
        <div className="flex items-center gap-1.5">
          <StatusDot tone={statusTone} />
          <Text>{title}</Text>
          {/* `StatusDot` is decorative-only (see its own doc comment) — this is the visible
          label it still needs for screen readers, just not rendered as its own line. */}
          <span className="sr-only">{statusLabel}</span>
        </div>
        <Text size="xs" tone="muted">
          {description}
        </Text>
      </div>
      {onToggle !== undefined && (
        <Button disabled={disabled || installed === undefined} onClick={onToggle}>
          {actionLabel}
        </Button>
      )}
    </div>
  )
}

export default ServiceStatusCard
