type StatusTone = 'active' | 'error' | 'idle' | 'pending'

const TONE_CLASS: Record<StatusTone, string> = {
  active: 'bg-success',
  error: 'bg-accent',
  idle: 'bg-muted/40',
  pending: 'animate-pulse bg-info',
}

interface StatusDotProps {
  tone: StatusTone
}

const StatusDot = ({ tone }: StatusDotProps): React.JSX.Element => (
  <span aria-hidden="true" className={`inline-block size-2 rounded-full ${TONE_CLASS[tone]}`} />
)

export default StatusDot
