import { useCallback, useReducer } from 'react'
import type UseDaemonRestartResult from '@/settings/application/use-daemon-restart.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  restarting: boolean
}

const initialState: State = { error: undefined, restarting: false }

type Action = { type: 'restartStarted' } | { type: 'restartSucceeded' } | { message: string; type: 'restartFailed' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'restartStarted': {
      return { ...state, error: undefined, restarting: true }
    }
    case 'restartSucceeded': {
      return { ...state, error: undefined, restarting: false }
    }
    case 'restartFailed': {
      return { ...state, error: action.message, restarting: false }
    }
  }
}

interface RestartArgs {
  addr: string
  dispatch: React.Dispatch<Action>
  onRestarted: () => void
}

const restartDaemon = async ({ addr, dispatch, onRestarted }: RestartArgs): Promise<void> => {
  dispatch({ type: 'restartStarted' })
  // eslint-disable-next-line unicorn/no-null
  const result = await commands.daemonRestart(addr, null)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'restartFailed' })
    return
  }
  dispatch({ type: 'restartSucceeded' })
  onRestarted()
}

const useDaemonRestart = (addr: string | undefined, onRestarted: () => void): UseDaemonRestartResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const restart = useCallback(() => {
    if (addr === undefined) {
      return
    }
    void restartDaemon({ addr, dispatch, onRestarted })
  }, [addr, onRestarted])

  return { ...state, restart }
}

export default useDaemonRestart
