import DeviceSetup from '@/gate/pages/DeviceSetup.tsx'
import useDeviceSetupFlow from '@/gate/application/use-device-setup-flow.ts'

interface DeviceSetupConnectedProps {
  onComplete: () => void
}

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
