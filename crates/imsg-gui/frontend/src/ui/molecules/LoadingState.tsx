import Spinner from '@/ui/atoms/Spinner.tsx'

interface LoadingStateProps {
  message: string
}

const LoadingState = ({ message }: LoadingStateProps): React.JSX.Element => (
  <div className="flex flex-col items-center gap-3">
    <Spinner />
    <p className="text-sm text-muted">{message}</p>
  </div>
)

export default LoadingState
