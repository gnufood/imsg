import { useCallback, useEffect, useReducer } from 'react'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import { commands } from '@/bindings.ts'

type Level = SecurityLevelArgs['draft']

const UNSET_FALLBACK: Level = 'Sdp'

interface State {
  draft: Level
  saveError: string | undefined
  saving: boolean
}

const initialState: State = { draft: UNSET_FALLBACK, saveError: undefined, saving: false }

type Action =
  | { level: Level; type: 'committedReceived' }
  | { draft: Level; type: 'draftChanged' }
  | { type: 'saveStarted' }
  | { type: 'saveSucceeded' }
  | { message: string; type: 'saveFailed' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'committedReceived': {
      return { ...state, draft: action.level, saveError: undefined }
    }
    case 'draftChanged': {
      return { ...state, draft: action.draft }
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
  }
}

interface SaveArgs {
  dispatch: React.Dispatch<Action>
  level: Level
  onSaved: () => void
}

const saveLevel = async ({ dispatch, level, onSaved }: SaveArgs): Promise<void> => {
  dispatch({ type: 'saveStarted' })
  const result = await commands.configSetBrokerSecurityLevel(level)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'saveFailed' })
    return
  }
  dispatch({ type: 'saveSucceeded' })
  onSaved()
}

interface CommittedArgs {
  committedLevel: Level | null | undefined
  dispatch: React.Dispatch<Action>
}

const useCommittedSync = ({ committedLevel, dispatch }: CommittedArgs): void => {
  useEffect(() => {
    if (committedLevel === undefined) {
      return
    }
    dispatch({ level: committedLevel ?? UNSET_FALLBACK, type: 'committedReceived' })
  }, [committedLevel, dispatch])
}

const useCancelAction = ({ committedLevel, dispatch }: CommittedArgs): (() => void) =>
  useCallback(() => {
    if (committedLevel === undefined) {
      return
    }
    dispatch({ level: committedLevel ?? UNSET_FALLBACK, type: 'committedReceived' })
  }, [committedLevel, dispatch])

interface UseSecurityLevelArgs {
  committedLevel: Level | null | undefined
  onSaved: () => void
}

const useSecurityLevel = ({ committedLevel, onSaved }: UseSecurityLevelArgs): SecurityLevelArgs => {
  const [state, dispatch] = useReducer(reduce, initialState)

  useCommittedSync({ committedLevel, dispatch })
  const onCancel = useCancelAction({ committedLevel, dispatch })

  const onSave = useCallback(() => {
    void saveLevel({ dispatch, level: state.draft, onSaved })
  }, [state.draft, onSaved])

  const onDraftChange = useCallback((draft: Level) => {
    dispatch({ draft, type: 'draftChanged' })
  }, [])

  return { committedLevel, draft: state.draft, onCancel, onDraftChange, onSave, saveError: state.saveError, saving: state.saving }
}

export default useSecurityLevel
