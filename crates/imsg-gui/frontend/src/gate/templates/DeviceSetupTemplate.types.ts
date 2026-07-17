import type { PairedDeviceDto } from '@/bindings.ts'

type DeviceSetupStage =
  | 'listing'
  | 'listError'
  | 'picking'
  | 'resolving'
  | 'resolveError'
  | 'unsupported'
  | 'persisting'
  | 'persistError'

export default interface DeviceSetupState {
  address: string | undefined
  devices: PairedDeviceDto[]
  errorMessage: string | undefined
  mapChannel: number | undefined
  pbapChannel: number | undefined
  stage: DeviceSetupStage
}
