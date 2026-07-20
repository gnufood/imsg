import { useCallback, useEffect, useReducer } from 'react'
import type UseChannelOverridesResult from '@/settings/application/use-channel-overrides.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  detectError: string | undefined
  detecting: boolean
  mapDraft: string
  pbapDraft: string
  saveError: string | undefined
  saving: boolean
}

const initialState: State = { detectError: undefined, detecting: false, mapDraft: '', pbapDraft: '', saveError: undefined, saving: false }

type Action =
  | { mapChannel: number; pbapChannel: number; type: 'committedReceived' }
  | { draft: string; type: 'mapDraftChanged' }
  | { draft: string; type: 'pbapDraftChanged' }
  | { type: 'saveStarted' }
  | { type: 'saveSucceeded' }
  | { message: string; type: 'saveFailed' }
  | { type: 'detectStarted' }
  | { mapChannel: number; pbapChannel: number; type: 'detectSucceeded' }
  | { message: string; type: 'detectFailed' }

// Pure — every transition names the state it lands on explicitly. `committedReceived` also backs
// `cancel` (see below), not just the initial-load effect — both want the same "drop whatever's
// In the drafts, reset to the last known-good values" behavior, errors included.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'committedReceived': {
      return { ...state, detectError: undefined, mapDraft: String(action.mapChannel), pbapDraft: String(action.pbapChannel), saveError: undefined }
    }
    case 'mapDraftChanged': {
      return { ...state, mapDraft: action.draft }
    }
    case 'pbapDraftChanged': {
      return { ...state, pbapDraft: action.draft }
    }
    case 'saveStarted': {
      return { ...state, saveError: undefined, saving: true }
    }
    case 'saveSucceeded': {
      return { ...state, saving: false }
    }
    case 'saveFailed': {
      return { ...state, saveError: action.message, saving: false }
    }
    case 'detectStarted': {
      return { ...state, detectError: undefined, detecting: true }
    }
    case 'detectSucceeded': {
      return { ...state, detecting: false, mapDraft: String(action.mapChannel), pbapDraft: String(action.pbapChannel) }
    }
    case 'detectFailed': {
      return { ...state, detectError: action.message, detecting: false }
    }
  }
}

interface SaveArgs {
  dispatch: React.Dispatch<Action>
  mapChannel: number
  onSaved: () => void
  pbapChannel: number
}

// Best-effort, same convention as `use-delete-conversation.ts`: fires both writes concurrently
// And surfaces one error if either failed, rather than aborting on the first failure.
const saveChannels = async ({ dispatch, mapChannel, onSaved, pbapChannel }: SaveArgs): Promise<void> => {
  dispatch({ type: 'saveStarted' })
  const results = await Promise.all([commands.configSetMapChannel(mapChannel), commands.configSetPbapChannel(pbapChannel)])
  const failed = results.find((result) => result.status === 'error')
  if (failed !== undefined && failed.status === 'error') {
    dispatch({ message: failed.error.message, type: 'saveFailed' })
    return
  }
  dispatch({ type: 'saveSucceeded' })
  onSaved()
}

interface DetectArgs {
  address: string
  dispatch: React.Dispatch<Action>
}

// Re-runs the same SDP lookup the device-setup gate does on first run (see
// `gate/application/use-device-setup-flow.ts`'s `resolveChannels`) and fills the drafts with the
// Result — doesn't persist it. A missing channel is treated as a failure here (unlike the gate
// Flow's dedicated "device unsupported" stage) since there's no equivalent stage to route to
// Mid-edit; the existing drafts are left untouched so a partial SDP response can't clobber them.
const detectChannels = async ({ address, dispatch }: DetectArgs): Promise<void> => {
  dispatch({ type: 'detectStarted' })
  const result = await commands.discoverResolveChannels(address)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'detectFailed' })
    return
  }
  if (typeof result.data.map !== 'number' || typeof result.data.pbap !== 'number') {
    dispatch({ message: "Device didn't report both MAP and PBAP channels.", type: 'detectFailed' })
    return
  }
  dispatch({ mapChannel: result.data.map, pbapChannel: result.data.pbap, type: 'detectSucceeded' })
}

interface CommittedArgs {
  dispatch: React.Dispatch<Action>
  mapChannel: number | undefined
  pbapChannel: number | undefined
}

// Re-seeds the drafts from the committed config values whenever they change — initial load, and
// Again after `onSaved` triggers a `reload`.
const useCommittedSync = ({ dispatch, mapChannel, pbapChannel }: CommittedArgs): void => {
  useEffect(() => {
    if (mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    dispatch({ mapChannel, pbapChannel, type: 'committedReceived' })
  }, [dispatch, mapChannel, pbapChannel])
}

// Discards whatever's in the drafts and falls back to the same `committedReceived` transition
// The initial-load effect uses — the only way out of `ChannelOverridesForm`'s editor besides
// Persisting (see that component's `closeEditor`).
const useCancelAction = ({ dispatch, mapChannel, pbapChannel }: CommittedArgs): (() => void) =>
  useCallback(() => {
    if (mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    dispatch({ mapChannel, pbapChannel, type: 'committedReceived' })
  }, [dispatch, mapChannel, pbapChannel])

interface DetectActionArgs {
  address: string | undefined
  dispatch: React.Dispatch<Action>
}

const useDetectAction = ({ address, dispatch }: DetectActionArgs): (() => void) =>
  useCallback(() => {
    if (address === undefined) {
      dispatch({ message: 'Device address unavailable.', type: 'detectFailed' })
      return
    }
    void detectChannels({ address, dispatch })
  }, [address, dispatch])

interface SaveActionArgs {
  dispatch: React.Dispatch<Action>
  mapDraft: string
  onSaved: () => void
  pbapDraft: string
}

const useSaveAction = ({ dispatch, mapDraft, onSaved, pbapDraft }: SaveActionArgs): (() => void) =>
  useCallback(() => {
    // `Number('')` is `0`, not `NaN` — checked separately so a blank draft doesn't silently
    // Save as channel 0.
    const mapBlank = mapDraft.trim() === ''
    const pbapBlank = pbapDraft.trim() === ''
    const parsedMap = Math.trunc(Number(mapDraft))
    const parsedPbap = Math.trunc(Number(pbapDraft))
    if (mapBlank || pbapBlank || Number.isNaN(parsedMap) || Number.isNaN(parsedPbap)) {
      dispatch({ message: 'Channels must be numbers.', type: 'saveFailed' })
      return
    }
    void saveChannels({ dispatch, mapChannel: parsedMap, onSaved, pbapChannel: parsedPbap })
  }, [dispatch, mapDraft, onSaved, pbapDraft])

interface UseChannelOverridesArgs {
  address: string | undefined
  mapChannel: number | undefined
  onSaved: () => void
  pbapChannel: number | undefined
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `configSetMapChannel`/`configSetPbapChannel`/
// `discoverResolveChannels`.
const useChannelOverrides = ({ address, mapChannel, onSaved, pbapChannel }: UseChannelOverridesArgs): UseChannelOverridesResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  useCommittedSync({ dispatch, mapChannel, pbapChannel })
  const cancel = useCancelAction({ dispatch, mapChannel, pbapChannel })
  const detect = useDetectAction({ address, dispatch })
  const save = useSaveAction({ dispatch, mapDraft: state.mapDraft, onSaved, pbapDraft: state.pbapDraft })

  const onMapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'mapDraftChanged' })
  }, [])

  const onPbapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'pbapDraftChanged' })
  }, [])

  return { ...state, cancel, detect, onMapDraftChange, onPbapDraftChange, save }
}

export default useChannelOverrides
