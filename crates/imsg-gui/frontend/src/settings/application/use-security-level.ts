import { useCallback, useEffect, useReducer } from 'react'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import { commands } from '@/bindings.ts'

// Reuses `SecurityLevelArgs`' own field type instead of importing `SecurityLevelDto` from
// `bindings.ts` directly — keeps this file's only `bindings.ts` import to `commands`, same
// Convention as every other hook here (see `use-config.types.ts` etc. for the DTO-type side).
type Level = SecurityLevelArgs['draft']

// `config::set_broker_security_level` always writes one of the four tiers — there's no "unset"
// Entry in `SecurityLevelDto` to fall back to, so a never-configured device (committed `null`)
// Seeds the draft here instead.
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

// Pure — every transition names the state it lands on explicitly.
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

// Re-seeds the draft from the committed config value whenever it changes — initial load, and
// Again after `onSaved` triggers a `reload`.
const useCommittedSync = ({ committedLevel, dispatch }: CommittedArgs): void => {
  useEffect(() => {
    if (committedLevel === undefined) {
      return
    }
    dispatch({ level: committedLevel ?? UNSET_FALLBACK, type: 'committedReceived' })
  }, [committedLevel, dispatch])
}

// Discards whatever's in the draft and falls back to the same `committedReceived` transition the
// Initial-load effect uses — the only way out of `SecurityLevelForm`'s editor besides persisting
// (See that component's `closeEditor`).
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

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `configSetBrokerSecurityLevel`. Returns
// `SecurityLevelArgs` directly (echoing `committedLevel` back out) instead of a separate
// Hook-result type plus a `SettingsConnected`-side `useMemo` — same reason `use-daemon-actions.ts`
// Composes straight into `DaemonControlsArgs` (avoids tripping `max-statements` (10) there).
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
