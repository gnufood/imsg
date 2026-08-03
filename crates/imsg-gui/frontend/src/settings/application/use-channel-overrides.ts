import { useCallback, useEffect, useReducer } from 'react'
import type UseChannelOverridesResult from '@/settings/application/use-channel-overrides.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  mapDraft: string
  pbapDraft: string
  saving: boolean
}

const initialState: State = { error: undefined, mapDraft: '', pbapDraft: '', saving: false }

type Action =
  | { mapChannel: number; pbapChannel: number; type: 'committedReceived' }
  | { draft: string; type: 'mapDraftChanged' }
  | { draft: string; type: 'pbapDraftChanged' }
  | { type: 'saveStarted' }
  | { type: 'saveSucceeded' }
  | { message: string; type: 'saveFailed' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'committedReceived': {
      return { ...state, mapDraft: String(action.mapChannel), pbapDraft: String(action.pbapChannel) }
    }
    case 'mapDraftChanged': {
      return { ...state, mapDraft: action.draft }
    }
    case 'pbapDraftChanged': {
      return { ...state, pbapDraft: action.draft }
    }
    case 'saveStarted': {
      return { ...state, error: undefined, saving: true }
    }
    case 'saveSucceeded': {
      return { ...state, saving: false }
    }
    case 'saveFailed': {
      return { ...state, error: action.message, saving: false }
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

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `configSetMapChannel`/`configSetPbapChannel`.
// `mapChannel`/`pbapChannel` are the committed values from `use-config.ts`; they re-seed the
// Drafts whenever they change — initial load, and again after `onSaved` triggers a `reload`.
const useChannelOverrides = (
  mapChannel: number | undefined,
  pbapChannel: number | undefined,
  onSaved: () => void,
): UseChannelOverridesResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  useEffect(() => {
    if (mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    dispatch({ mapChannel, pbapChannel, type: 'committedReceived' })
  }, [mapChannel, pbapChannel])

  const onMapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'mapDraftChanged' })
  }, [])

  const onPbapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'pbapDraftChanged' })
  }, [])

  const save = useCallback(() => {
    // `Number('')` is `0`, not `NaN` — checked separately so a blank draft doesn't silently
    // Save as channel 0.
    const mapBlank = state.mapDraft.trim() === ''
    const pbapBlank = state.pbapDraft.trim() === ''
    const parsedMap = Math.trunc(Number(state.mapDraft))
    const parsedPbap = Math.trunc(Number(state.pbapDraft))
    if (mapBlank || pbapBlank || Number.isNaN(parsedMap) || Number.isNaN(parsedPbap)) {
      dispatch({ message: 'Channels must be numbers.', type: 'saveFailed' })
      return
    }
    void saveChannels({ dispatch, mapChannel: parsedMap, onSaved, pbapChannel: parsedPbap })
  }, [state.mapDraft, state.pbapDraft, onSaved])

  return { ...state, onMapDraftChange, onPbapDraftChange, save }
}

export default useChannelOverrides
