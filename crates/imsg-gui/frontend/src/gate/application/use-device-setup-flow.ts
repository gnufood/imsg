import { useCallback, useEffect, useReducer } from 'react'
import type UseDeviceSetupFlowResult from '@/gate/application/use-device-setup-flow.types.ts'
import { commands } from '@/bindings.ts'

type DeviceSetupState = UseDeviceSetupFlowResult['state']
type DeviceSetupStage = DeviceSetupState['stage']

const initialState: DeviceSetupState = {
  address: undefined,
  devices: [],
  errorMessage: undefined,
  mapChannel: undefined,
  pbapChannel: undefined,
  stage: 'listing',
}

// 'retry' folds the three identical "clear the error, re-enter a fetch stage" transitions
// (list/resolve/persist) into one case — keeps `reduce` under this repo's max-statements limit.
type Action =
  | { stage: 'listing' | 'persisting' | 'resolving'; type: 'retry' }
  | { devices: DeviceSetupState['devices']; type: 'devicesLoaded' }
  | { message: string; type: 'listFailed' }
  | { address: string; type: 'deviceSelected' }
  | { mapChannel: number; pbapChannel: number; type: 'channelsResolved' }
  | { type: 'deviceUnsupported' }
  | { message: string; type: 'resolveFailed' }
  | { message: string; type: 'persistFailed' }

// Pure — every transition names the stage it lands on explicitly, no derived/implicit stage.
const reduce = (state: DeviceSetupState, action: Action): DeviceSetupState => {
  switch (action.type) {
    case 'retry': {
      return { ...state, errorMessage: undefined, stage: action.stage }
    }
    case 'devicesLoaded': {
      return { ...state, devices: action.devices, stage: 'picking' }
    }
    case 'listFailed': {
      return { ...state, errorMessage: action.message, stage: 'listError' }
    }
    case 'deviceSelected': {
      return { ...state, address: action.address, stage: 'resolving' }
    }
    case 'channelsResolved': {
      return { ...state, mapChannel: action.mapChannel, pbapChannel: action.pbapChannel, stage: 'persisting' }
    }
    case 'deviceUnsupported': {
      return { ...state, stage: 'unsupported' }
    }
    case 'resolveFailed': {
      return { ...state, errorMessage: action.message, stage: 'resolveError' }
    }
    case 'persistFailed': {
      return { ...state, errorMessage: action.message, stage: 'persistError' }
    }
  }
}

interface LoadDevicesArgs {
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

const loadDevices = async ({ cancelled, dispatch }: LoadDevicesArgs): Promise<void> => {
  const result = await commands.discoverListPairedDevices()
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ devices: result.data, type: 'devicesLoaded' })
    return
  }
  dispatch({ message: result.error.message, type: 'listFailed' })
}

interface ResolveChannelsArgs {
  address: string
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

const resolveChannels = async ({ address, cancelled, dispatch }: ResolveChannelsArgs): Promise<void> => {
  const result = await commands.discoverResolveChannels(address)
  if (cancelled.current) {
    return
  }
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'resolveFailed' })
    return
  }
  // A missing channel is a real device that just doesn't run that service — a success-path
  // Case (see GUI_FRONTEND.md's "DeviceSetup's error handling"), checked via `typeof` rather
  // Than a `null` comparison per this repo's `unicorn/no-null` lint rule.
  if (typeof result.data.map !== 'number' || typeof result.data.pbap !== 'number') {
    dispatch({ type: 'deviceUnsupported' })
    return
  }
  dispatch({ mapChannel: result.data.map, pbapChannel: result.data.pbap, type: 'channelsResolved' })
}

interface PersistDeviceArgs {
  address: string
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
  mapChannel: number
  onComplete: () => void
  pbapChannel: number
}

const persistDevice = async ({
  address,
  cancelled,
  dispatch,
  mapChannel,
  onComplete,
  pbapChannel,
}: PersistDeviceArgs): Promise<void> => {
  const result = await commands.configSetDeviceAndChannels(address, mapChannel, pbapChannel)
  if (cancelled.current) {
    return
  }
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'persistFailed' })
    return
  }
  onComplete()
}

const useLoadDevices = (stage: DeviceSetupStage, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (stage !== 'listing') {
      return
    }
    const cancelled = { current: false }
    void loadDevices({ cancelled, dispatch })
    return () => {
      cancelled.current = true
    }
  }, [stage, dispatch])
}

const useResolveChannels = (stage: DeviceSetupStage, address: string | undefined, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (stage !== 'resolving' || address === undefined) {
      return
    }
    const cancelled = { current: false }
    void resolveChannels({ address, cancelled, dispatch })
    return () => {
      cancelled.current = true
    }
  }, [stage, address, dispatch])
}

interface UsePersistDeviceArgs {
  address: string | undefined
  dispatch: React.Dispatch<Action>
  mapChannel: number | undefined
  onComplete: () => void
  pbapChannel: number | undefined
  stage: DeviceSetupStage
}

const usePersistDevice = ({ address, dispatch, mapChannel, onComplete, pbapChannel, stage }: UsePersistDeviceArgs): void => {
  useEffect(() => {
    if (stage !== 'persisting' || address === undefined || mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    const cancelled = { current: false }
    void persistDevice({ address, cancelled, dispatch, mapChannel, onComplete, pbapChannel })
    return () => {
      cancelled.current = true
    }
  }, [stage, address, mapChannel, pbapChannel, dispatch, onComplete])
}

// Application boundary for the device-setup feature slice (see internal/GUI_ATOMIC_DESIGN.md)
// — the only file here allowed to import `bindings.ts`.
const useDeviceSetupFlow = (onComplete: () => void): UseDeviceSetupFlowResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { stage, address, mapChannel, pbapChannel } = state

  const retryList = useCallback(() => {
    dispatch({ stage: 'listing', type: 'retry' })
  }, [])

  const selectDevice = useCallback((selected: string) => {
    dispatch({ address: selected, type: 'deviceSelected' })
  }, [])

  const retryResolve = useCallback(() => {
    dispatch({ stage: 'resolving', type: 'retry' })
  }, [])

  const retryPersist = useCallback(() => {
    dispatch({ stage: 'persisting', type: 'retry' })
  }, [])

  useLoadDevices(stage, dispatch)
  useResolveChannels(stage, address, dispatch)
  usePersistDevice({ address, dispatch, mapChannel, onComplete, pbapChannel, stage })

  return { retryList, retryPersist, retryResolve, selectDevice, state }
}

export default useDeviceSetupFlow
