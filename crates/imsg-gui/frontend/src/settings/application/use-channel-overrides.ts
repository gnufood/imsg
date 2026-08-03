import { useCallback, useEffect, useReducer } from 'react'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  detectError: string | undefined
  detecting: boolean
  editorOpen: boolean
  mapDraft: string
  pbapDraft: string
  saveError: string | undefined
  saving: boolean
}

const initialState: State = {
  detectError: undefined,
  detecting: false,
  editorOpen: false,
  mapDraft: '',
  pbapDraft: '',
  saveError: undefined,
  saving: false,
}

type EditorAction =
  | { mapChannel: number; pbapChannel: number; type: 'committedReceived' }
  | { draft: string; type: 'mapDraftChanged' }
  | { draft: string; type: 'pbapDraftChanged' }
  | { type: 'editorOpened' }
  | { mapChannel: number; pbapChannel: number; type: 'editorCancelled' }

type RequestAction =
  | { type: 'saveStarted' }
  | { type: 'saveSucceeded' }
  | { message: string; type: 'saveFailed' }
  | { type: 'detectStarted' }
  | { mapChannel: number; pbapChannel: number; type: 'detectSucceeded' }
  | { message: string; type: 'detectFailed' }

type Action = EditorAction | RequestAction

const reseeded = (state: State, mapChannel: number, pbapChannel: number): State => ({
  ...state,
  detectError: undefined,
  mapDraft: String(mapChannel),
  pbapDraft: String(pbapChannel),
  saveError: undefined,
})

const reduceRequest = (state: State, action: RequestAction): State => {
  switch (action.type) {
    case 'saveStarted': {
      return { ...state, saveError: undefined, saving: true }
    }
    case 'saveSucceeded': {
      return { ...state, editorOpen: false, saving: false }
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

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'committedReceived': {
      return reseeded(state, action.mapChannel, action.pbapChannel)
    }
    case 'editorOpened': {
      return { ...state, editorOpen: true }
    }
    case 'editorCancelled': {
      return { ...reseeded(state, action.mapChannel, action.pbapChannel), editorOpen: false }
    }
    case 'mapDraftChanged': {
      return { ...state, mapDraft: action.draft }
    }
    case 'pbapDraftChanged': {
      return { ...state, pbapDraft: action.draft }
    }
    default: {
      return reduceRequest(state, action)
    }
  }
}

interface SaveArgs {
  dispatch: React.Dispatch<Action>
  mapChannel: number
  onSaved: () => void
  pbapChannel: number
}

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

const useCommittedSync = ({ dispatch, mapChannel, pbapChannel }: CommittedArgs): void => {
  useEffect(() => {
    if (mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    dispatch({ mapChannel, pbapChannel, type: 'committedReceived' })
  }, [dispatch, mapChannel, pbapChannel])
}

const useCancelAction = ({ dispatch, mapChannel, pbapChannel }: CommittedArgs): (() => void) =>
  useCallback(() => {
    if (mapChannel === undefined || pbapChannel === undefined) {
      return
    }
    dispatch({ mapChannel, pbapChannel, type: 'editorCancelled' })
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

const useChannelOverrides = ({ address, mapChannel, onSaved, pbapChannel }: UseChannelOverridesArgs): ChannelOverridesArgs => {
  const [state, dispatch] = useReducer(reduce, initialState)

  useCommittedSync({ dispatch, mapChannel, pbapChannel })
  const onCancel = useCancelAction({ dispatch, mapChannel, pbapChannel })
  const onDetect = useDetectAction({ address, dispatch })
  const onSave = useSaveAction({ dispatch, mapDraft: state.mapDraft, onSaved, pbapDraft: state.pbapDraft })

  const onRequestEdit = useCallback(() => {
    dispatch({ type: 'editorOpened' })
  }, [])

  const onMapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'mapDraftChanged' })
  }, [])

  const onPbapDraftChange = useCallback((draft: string) => {
    dispatch({ draft, type: 'pbapDraftChanged' })
  }, [])

  return { ...state, mapChannel, onCancel, onDetect, onMapDraftChange, onPbapDraftChange, onRequestEdit, onSave, pbapChannel }
}

export default useChannelOverrides
