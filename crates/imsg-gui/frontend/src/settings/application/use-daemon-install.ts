import { useCallback, useReducer } from 'react'
import type UseDaemonInstallResult from '@/settings/application/use-daemon-install.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  installing: boolean
}

const initialState: State = { error: undefined, installing: false }

type Action = { type: 'installStarted' } | { type: 'installSucceeded' } | { message: string; type: 'installFailed' }

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
}

const installDaemon = async ({ addr, dispatch, onInstalled }: InstallArgs): Promise<void> => {
  dispatch({ type: 'installStarted' })
  // eslint-disable-next-line unicorn/no-null
  const result = await commands.daemonInstall(addr, null, false)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'installFailed' })
    return
  }
  dispatch({ type: 'installSucceeded' })
  onInstalled()
}

const useDaemonInstall = (addr: string | undefined, onInstalled: () => void): UseDaemonInstallResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const install = useCallback(() => {
    if (addr === undefined) {
      return
    }
    void installDaemon({ addr, dispatch, onInstalled })
  }, [addr, onInstalled])

  return { ...state, install }
}

export default useDaemonInstall
