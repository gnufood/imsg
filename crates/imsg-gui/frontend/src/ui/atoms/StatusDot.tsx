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

// Decorative only — the caller always renders a visible text label alongside it (see
// `StatusPanel`), same convention as `Avatar`. Success/info (index.css) are status-only tokens —
// Distinct hues per tone, unlike ink/muted which read as shades of the same color.
const StatusDot = ({ tone }: StatusDotProps): React.JSX.Element => (
  <span aria-hidden="true" className={`inline-block size-2 rounded-full ${TONE_CLASS[tone]}`} />
)

export default StatusDot
