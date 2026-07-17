import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import type { GateStatus } from '@/gate/application/use-gate-status.ts'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import Preparing from '@/gate/organisms/Preparing.tsx'
import Splash from '@/gate/organisms/Splash.tsx'

type PreparingStage = Parameters<typeof Preparing>[0]['stage']

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

const failedStage = (stage: 'Daemon' | 'Sync'): PreparingStage => {
  if (stage === 'Daemon') {
    return 'daemon'
  }
  return 'sync'
}

const renderStatus = (status: GateStatus | undefined, deviceSetupSlot: React.JSX.Element, onProceed: () => void): React.JSX.Element => {
  if (status === 'AwaitingDeviceConfig') {
    return deviceSetupSlot
  }
  if (status === 'StartingDaemon') {
    return <Preparing stage="daemon" onRetry={onProceed} />
  }
  if (status === 'Syncing') {
    return <Preparing stage="sync" onRetry={onProceed} />
  }
  if (status !== undefined && status !== 'Initializing' && status !== 'Ready') {
    return <Preparing stage={failedStage(status.Failed.stage)} error={status.Failed.message} onRetry={onProceed} />
  }
  // `undefined` (first poll in flight), 'Initializing', or 'Ready' — the last is a single
  // Transitional frame (the handoff Effect in `Gate` fires on this same render pass, then the
  // Parent swaps Gate out), reusing the loading treatment avoids rendering "nothing" (forbidden
  // By `unicorn/no-null`/`react/jsx-no-useless-fragment`).
  return screen(<LoadingState message="Starting up…" />)
}

interface GateTemplateProps {
  // Filled by the composition root with a connected component (e.g. `DeviceSetupConnected`)
  // When `status` is `'AwaitingDeviceConfig'` — keeps this template IPC-agnostic and
  // Renderable from fixtures for every other status.
  deviceSetupSlot: React.JSX.Element
  onProceed: () => void
  onResumePolling: () => void
  onSplashDone: () => void
  pollFailed: boolean
  splashDone: boolean
  status: GateStatus | undefined
}

const GateTemplate = ({
  deviceSetupSlot,
  onProceed,
  onResumePolling,
  onSplashDone,
  pollFailed,
  splashDone,
  status,
}: GateTemplateProps): React.JSX.Element => {
  if (!splashDone) {
    return <Splash ready={status !== undefined || pollFailed} onDone={onSplashDone} />
  }
  if (pollFailed) {
    return screen(<ErrorState message="Couldn't reach the app backend." onRetry={onResumePolling} />)
  }
  return renderStatus(status, deviceSetupSlot, onProceed)
}

export default GateTemplate
