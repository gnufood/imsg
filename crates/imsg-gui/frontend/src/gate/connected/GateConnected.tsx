import DeviceSetupConnected from '@/gate/connected/DeviceSetupConnected.tsx'
import Gate from '@/gate/pages/Gate.tsx'
import useGateStatus from '@/gate/application/use-gate-status.ts'
import { useMemo } from 'react'

interface GateConnectedProps {
  onReady: () => void
}

const GateConnected = ({ onReady }: GateConnectedProps): React.JSX.Element => {
  const { pollFailed, proceed, resumePolling, status } = useGateStatus()

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
