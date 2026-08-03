import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'

type PreparingStage = 'daemon' | 'sync'

interface PreparingProps {
  stage: PreparingStage
  error?: string
  onRetry: () => void
}

const STAGE_MESSAGE: Record<PreparingStage, string> = {
  daemon: 'Starting the daemon…',
  sync: 'Syncing messages…',
}

const Preparing = ({ stage, error, onRetry }: PreparingProps): React.JSX.Element => (
  <CenteredScreen>
    {error === undefined && <LoadingState message={STAGE_MESSAGE[stage]} />}
    {error !== undefined && <ErrorState message={error} onRetry={onRetry} />}
  </CenteredScreen>
)

export default Preparing
