type StatusTone = 'active' | 'error' | 'idle' | 'pending'

const TONE_CLASS: Record<StatusTone, string> = {
  active: 'bg-ink',
  error: 'bg-accent',
  idle: 'bg-muted/40',
  pending: 'animate-pulse bg-muted',
}

interface StatusDotProps {
  tone: StatusTone
}

// Decorative only — the caller always renders a visible text label alongside it (see
// `StatusPanel`), same convention as `Avatar`. Sticks to the existing 5-token palette
// (ink/accent/muted) rather than introducing new status colors.
const StatusDot = ({ tone }: StatusDotProps): React.JSX.Element => (
  <span aria-hidden="true" className={`inline-block size-2 rounded-full ${TONE_CLASS[tone]}`} />
)

export default StatusDot
