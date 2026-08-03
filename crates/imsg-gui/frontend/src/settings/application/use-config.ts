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
