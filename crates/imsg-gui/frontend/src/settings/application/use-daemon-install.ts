import { useCallback, useReducer } from 'react'
import type UseDaemonInstallResult from '@/settings/application/use-daemon-install.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  installing: boolean
}

const initialState: State = { error: undefined, installing: false }

type Action = { type: 'installStarted' } | { type: 'installSucceeded' } | { message: string; type: 'installFailed' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'installStarted': {
      return { ...state, error: undefined, installing: true }
    }
    case 'installSucceeded': {
      return { ...state, error: undefined, installing: false }
    }
    case 'installFailed': {
      return { ...state, error: action.message, installing: false }
    }
  }
}

interface InstallArgs {
  addr: string
  dispatch: React.Dispatch<Action>
  onInstalled: () => void
  system: boolean
}

const installDaemon = async ({ addr, dispatch, onInstalled, system }: InstallArgs): Promise<void> => {
  dispatch({ type: 'installStarted' })
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const result = await commands.daemonInstall(addr, null, system)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'installFailed' })
    return
  }
  dispatch({ type: 'installSucceeded' })
  onInstalled()
}

// Application boundary for the settings feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `daemonInstall`. `addr` is `undefined` until
// `use-config.ts` resolves the device address, in which case `install` is a no-op.
const useDaemonInstall = (addr: string | undefined, onInstalled: () => void): UseDaemonInstallResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const install = useCallback(
    (system: boolean) => {
      if (addr === undefined) {
        return
      }
      void installDaemon({ addr, dispatch, onInstalled, system })
    },
    [addr, onInstalled],
  )

  return { ...state, install }
}

export default useDaemonInstall
