import DeviceSetup from '@/gate/pages/DeviceSetup.tsx'
import useDeviceSetupFlow from '@/gate/application/use-device-setup-flow.ts'

interface DeviceSetupConnectedProps {
  onComplete: () => void
}

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useDeviceSetupFlow`'s real backend calls and `DeviceSetup`'s presentational page.
const DeviceSetupConnected = ({ onComplete }: DeviceSetupConnectedProps): React.JSX.Element => {
  const { retryList, retryPersist, retryResolve, selectDevice, state } = useDeviceSetupFlow(onComplete)

  return (
    <DeviceSetup
      onRetryList={retryList}
      onRetryPersist={retryPersist}
      onRetryResolve={retryResolve}
      onSelectDevice={selectDevice}
      state={state}
    />
  )
}

export default DeviceSetupConnected
