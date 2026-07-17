import DeviceSetupConnected from '@/gate/connected/DeviceSetupConnected.tsx'
import Gate from '@/gate/pages/Gate.tsx'
import { useGateStatus } from '@/gate/application/use-gate-status.ts'
import { useMemo } from 'react'

interface GateConnectedProps {
  onReady: () => void
}

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useGateStatus`'s real backend polling and `Gate`'s presentational page.
const GateConnected = ({ onReady }: GateConnectedProps): React.JSX.Element => {
  const { pollFailed, proceed, resumePolling, status } = useGateStatus()

  // `proceed`'s identity is stable (empty-deps `useCallback` in `useGateStatus`), so this only
  // Actually recreates once — avoids remounting `DeviceSetupConnected` on every 250ms poll tick.
  const deviceSetupSlot = useMemo(() => <DeviceSetupConnected onComplete={proceed} />, [proceed])

  return (
    <Gate
      deviceSetupSlot={deviceSetupSlot}
      onProceed={proceed}
      onReady={onReady}
      onResumePolling={resumePolling}
      pollFailed={pollFailed}
      status={status}
    />
  )
}

export default GateConnected
