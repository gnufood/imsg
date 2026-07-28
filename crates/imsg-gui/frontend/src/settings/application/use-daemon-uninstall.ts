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

// Pure — every transition names the state it lands on explicitly.
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
  system: boolean
}

const uninstallDaemon = async ({ dispatch, onUninstalled, system }: UninstallArgs): Promise<void> => {
  dispatch({ type: 'uninstallStarted' })
  const result = await commands.daemonUninstall(system)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'uninstallFailed' })
    return
  }
  dispatch({ outcome: result.data, type: 'uninstallSucceeded' })
  onUninstalled()
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `daemonUninstall`. Doesn't need the device
// Address — unlike stop/restart/install, `daemon_uninstall` only takes `system`.
const useDaemonUninstall = (onUninstalled: () => void): UseDaemonUninstallResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const uninstall = useCallback(
    (system: boolean) => {
      void uninstallDaemon({ dispatch, onUninstalled, system })
    },
    [onUninstalled],
  )

  return { ...state, uninstall }
}

export default useDaemonUninstall
