import Spinner from '@/ui/atoms/Spinner.tsx'
import Text from '@/ui/atoms/Text.tsx'

interface LoadingStateProps {
  message: string
}

const LoadingState = ({ message }: LoadingStateProps): React.JSX.Element => (
  <div className="flex flex-col items-center gap-3">
    <Spinner />
    <Text tone="muted">{message}</Text>
  </div>
)

export default LoadingState
