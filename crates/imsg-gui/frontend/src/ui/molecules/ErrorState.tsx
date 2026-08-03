import { CircleAlert, RotateCcw } from 'lucide-react'
import Button from '@/ui/atoms/Button.tsx'
import Text from '@/ui/atoms/Text.tsx'

interface ErrorStateProps {
  message: string
  onRetry: () => void
  icon?: typeof CircleAlert
}

const ErrorState = ({ message, onRetry, icon: Icon = CircleAlert }: ErrorStateProps): React.JSX.Element => (
  <div className="flex flex-col items-center gap-3">
    <Icon className="size-7 text-accent" aria-hidden="true" />
    <Text>{message}</Text>
    <Button onClick={onRetry} icon={RotateCcw}>
      Retry
    </Button>
  </div>
)

export default ErrorState
