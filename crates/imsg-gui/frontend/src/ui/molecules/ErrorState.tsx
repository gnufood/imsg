import { CircleAlert, RotateCcw } from 'lucide-react'
import Button from '@/ui/atoms/Button.tsx'

interface ErrorStateProps {
  message: string
  onRetry: () => void
  // Caller picks this — `CommandError` carries no error-kind discriminant (see bindings.ts).
  // Only the call site (which knows which command it invoked) can tell errors apart.
  // `typeof CircleAlert` (not a separate `LucideIcon` import) to avoid a duplicate-import from
  // 'lucide-react' — this file already imports it as a value for the default icon.
  icon?: typeof CircleAlert
}

const ErrorState = ({ message, onRetry, icon: Icon = CircleAlert }: ErrorStateProps): React.JSX.Element => (
  <div className="flex flex-col items-center gap-3">
    <Icon className="size-7 text-accent" aria-hidden="true" />
    <p className="text-sm text-ink">{message}</p>
    <Button onClick={onRetry} icon={RotateCcw}>
      Retry
    </Button>
  </div>
)

export default ErrorState
