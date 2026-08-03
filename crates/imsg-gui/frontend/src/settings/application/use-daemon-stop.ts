import { useCallback, useReducer } from 'react'
import type UseDaemonStopResult from '@/settings/application/use-daemon-stop.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  stopping: boolean
}

const initialState: State = { error: undefined, stopping: false }

type Action = { type: 'stopStarted' } | { type: 'stopSucceeded' } | { message: string; type: 'stopFailed' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'stopStarted': {
      return { ...state, error: undefined, stopping: true }
    }
    case 'stopSucceeded': {
      return { ...state, error: undefined, stopping: false }
    }
    case 'stopFailed': {
      return { ...state, error: action.message, stopping: false }
    }
  }
}

interface StopArgs {
  addr: string
  dispatch: React.Dispatch<Action>
  onStopped: () => void
}

const stopDaemon = async ({ addr, dispatch, onStopped }: StopArgs): Promise<void> => {
  dispatch({ type: 'stopStarted' })
  const result = await commands.daemonStop(addr)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'stopFailed' })
    return
  }
  dispatch({ type: 'stopSucceeded' })
  onStopped()
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `daemonStop`. `addr` is `undefined` until
// `use-config.ts` resolves the device address, in which case `stop` is a no-op.
const useDaemonStop = (addr: string | undefined, onStopped: () => void): UseDaemonStopResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const stop = useCallback(() => {
    if (addr === undefined) {
      return
    }
    void stopDaemon({ addr, dispatch, onStopped })
  }, [addr, onStopped])

  return { ...state, stop }
}

export default useDaemonStop
