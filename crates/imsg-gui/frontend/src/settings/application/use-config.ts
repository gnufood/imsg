import { useCallback, useEffect, useReducer } from 'react'
import type UseConfigResult from '@/settings/application/use-config.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  config: UseConfigResult['config']
  failed: boolean
  version: number
}

const initialState: State = { config: undefined, failed: false, version: 0 }

type Action = { config: NonNullable<UseConfigResult['config']>; type: 'configLoaded' } | { type: 'loadFailed' } | { type: 'reload' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'configLoaded': {
      return { ...state, config: action.config, failed: false }
    }
    case 'loadFailed': {
      return { ...state, failed: true }
    }
    case 'reload': {
      return { ...state, failed: false, version: state.version + 1 }
    }
  }
}

interface LoadArgs {
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

const loadConfig = async ({ cancelled, dispatch }: LoadArgs): Promise<void> => {
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const result = await commands.configShow(null)
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ config: result.data, type: 'configLoaded' })
    return
  }
  dispatch({ type: 'loadFailed' })
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `configShow`. One-shot fetch, not polled —
// Config only changes through this screen's own edits, which call `reload` (bumping `version`,
// The effect's only dependency) rather than patching state optimistically.
const useConfig = (): UseConfigResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { config, failed, version } = state

  useEffect(() => {
    const cancelled = { current: false }
    void loadConfig({ cancelled, dispatch })
    return () => {
      cancelled.current = true
    }
  }, [version])

  const reload = useCallback(() => {
    dispatch({ type: 'reload' })
  }, [])

  return { config, failed, reload }
}

export default useConfig
