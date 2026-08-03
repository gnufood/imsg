import { useCallback, useReducer } from 'react'
import type UseDaemonUninstallResult from '@/settings/application/use-daemon-uninstall.types.ts'
import { commands } from '@/bindings.ts'

type Outcome = UseDaemonUninstallResult['outcome']

interface State {
  error: string | undefined
  outcome: Outcome
  uninstalling: boolean
}

const initialState: State = { error: undefined, outcome: undefined, uninstalling: false }

type Action =
  | { type: 'uninstallStarted' }
  | { outcome: NonNullable<Outcome>; type: 'uninstallSucceeded' }
  | { message: string; type: 'uninstallFailed' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'uninstallStarted': {
      return { ...state, error: undefined, uninstalling: true }
    }
    case 'uninstallSucceeded': {
      return { ...state, error: undefined, outcome: action.outcome, uninstalling: false }
    }
    case 'uninstallFailed': {
      return { ...state, error: action.message, uninstalling: false }
    }
  }
}

interface UninstallArgs {
  dispatch: React.Dispatch<Action>
  onUninstalled: () => void
}

const uninstallDaemon = async ({ dispatch, onUninstalled }: UninstallArgs): Promise<void> => {
  dispatch({ type: 'uninstallStarted' })
  const result = await commands.daemonUninstall(false)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'uninstallFailed' })
    return
  }
  dispatch({ outcome: result.data, type: 'uninstallSucceeded' })
  onUninstalled()
}

const useDaemonUninstall = (onUninstalled: () => void): UseDaemonUninstallResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const uninstall = useCallback(() => {
    void uninstallDaemon({ dispatch, onUninstalled })
  }, [onUninstalled])

  return { ...state, uninstall }
}

export default useDaemonUninstall
