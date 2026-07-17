import type { DeviceSetupState } from '@/gate/application/use-device-setup-flow.ts'
import DeviceSetupTemplate from '@/gate/templates/DeviceSetupTemplate.tsx'

interface DeviceSetupProps {
  onRetryList: () => void
  onRetryPersist: () => void
  onRetryResolve: () => void
  onSelectDevice: (address: string) => void
  state: DeviceSetupState
}

const DeviceSetup = ({
  onRetryList,
  onRetryPersist,
  onRetryResolve,
  onSelectDevice,
  state,
}: DeviceSetupProps): React.JSX.Element => (
  <DeviceSetupTemplate
    onRetryList={onRetryList}
    onRetryPersist={onRetryPersist}
    onRetryResolve={onRetryResolve}
    onSelectDevice={onSelectDevice}
    state={state}
  />
)

export default DeviceSetup
