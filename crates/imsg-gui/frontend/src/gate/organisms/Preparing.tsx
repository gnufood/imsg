import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'

// Mirrors GUI.md's revised startup sequence, steps 3 (`ensure_running`) and 4 (`ensure_synced`).
type PreparingStage = 'daemon' | 'sync'

interface PreparingProps {
  stage: PreparingStage
  // Set once `main.rs`'s event wiring (still unbuilt, see GUI_FRONTEND.md) reports a stage
  // Failure. `undefined` renders the loading branch.
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
