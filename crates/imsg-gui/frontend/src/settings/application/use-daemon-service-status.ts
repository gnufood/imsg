import { useCallback, useEffect, useReducer } from 'react'
import type UseDaemonServiceStatusResult from '@/settings/application/use-daemon-service-status.types.ts'
import { commands } from '@/bindings.ts'

const POLL_MS = 5000

interface State {
  pollFailed: boolean
  systemInstalled: boolean | undefined
  userInstalled: boolean | undefined
}

const initialState: State = { pollFailed: false, systemInstalled: undefined, userInstalled: undefined }

type Action =
  | { systemInstalled: boolean; type: 'statusReceived'; userInstalled: boolean }
  | { type: 'pollFailed' }
  | { type: 'resumePolling' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'statusReceived': {
      return { ...state, pollFailed: false, systemInstalled: action.systemInstalled, userInstalled: action.userInstalled }
    }
    case 'pollFailed': {
      return { ...state, pollFailed: true }
    }
    case 'resumePolling': {
      return { ...state, pollFailed: false }
    }
  }
}

interface PollArgs {
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

const pollServiceStatus = async ({ cancelled, dispatch }: PollArgs): Promise<void> => {
  const [user, system] = await Promise.all([commands.daemonServiceStatus(false), commands.daemonServiceStatus(true)])
  if (cancelled.current) {
    return
  }
  if (user.status === 'error' || system.status === 'error') {
    dispatch({ type: 'pollFailed' })
    return
  }
  dispatch({ systemInstalled: system.data !== 'NotInstalled', type: 'statusReceived', userInstalled: user.data !== 'NotInstalled' })
}

const usePollServiceStatus = (active: boolean, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (!active) {
      return
    }
    const cancelled = { current: false }
    void pollServiceStatus({ cancelled, dispatch })
    const interval = setInterval(() => {
      void pollServiceStatus({ cancelled, dispatch })
    }, POLL_MS)
    return () => {
      cancelled.current = true
      clearInterval(interval)
    }
  }, [active, dispatch])
}

const useDaemonServiceStatus = (): UseDaemonServiceStatusResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { pollFailed, systemInstalled, userInstalled } = state

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  usePollServiceStatus(!pollFailed, dispatch)

  return { pollFailed, resumePolling, systemInstalled, userInstalled }
}

export default useDaemonServiceStatus
