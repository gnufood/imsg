import { useCallback, useReducer } from 'react'
import type UseDaemonRestartResult from '@/settings/application/use-daemon-restart.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  restarting: boolean
}

const initialState: State = { error: undefined, restarting: false }

type Action = { type: 'restartStarted' } | { type: 'restartSucceeded' } | { message: string; type: 'restartFailed' }

// Pure — every transition names the state it lands on explicitly.
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
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const result = await commands.daemonRestart(addr, null)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'restartFailed' })
    return
  }
  dispatch({ type: 'restartSucceeded' })
  onRestarted()
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `daemonRestart`. `addr` is `undefined` until
// `use-config.ts` resolves the device address, in which case `restart` is a no-op.
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
