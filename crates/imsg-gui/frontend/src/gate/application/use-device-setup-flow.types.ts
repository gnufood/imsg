import type DeviceSetupState from '@/gate/templates/DeviceSetupTemplate.types.ts'

export default interface UseDeviceSetupFlowResult {
  retryList: () => void
  retryPersist: () => void
  retryResolve: () => void
  selectDevice: (address: string) => void
  state: DeviceSetupState
}
