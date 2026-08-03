import { useCallback, useEffect, useReducer } from 'react'
import type UseDaemonStatusResult from '@/settings/application/use-daemon-status.types.ts'
import { commands } from '@/bindings.ts'

const POLL_MS = 5000

type Status = UseDaemonStatusResult['status']
type ReceivedStatus = Exclude<Status, undefined>

interface State {
  pollFailed: boolean
  status: Status
}

const initialState: State = { pollFailed: false, status: undefined }

type Action = { status: ReceivedStatus; type: 'statusReceived' } | { type: 'pollFailed' } | { type: 'resumePolling' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'statusReceived': {
      return { ...state, pollFailed: false, status: action.status }
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
  addr: string
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

const pollStatus = async ({ addr, cancelled, dispatch }: PollArgs): Promise<void> => {
  try {
    const status = await commands.daemonStatus(addr)
    if (!cancelled.current) {
      dispatch({ status, type: 'statusReceived' })
    }
  } catch {
    if (!cancelled.current) {
      dispatch({ type: 'pollFailed' })
    }
  }
}

const usePollStatus = (addr: string | undefined, active: boolean, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (addr === undefined || !active) {
      return
    }
    const cancelled = { current: false }
    void pollStatus({ addr, cancelled, dispatch })
    const interval = setInterval(() => {
      void pollStatus({ addr, cancelled, dispatch })
    }, POLL_MS)
    return () => {
      cancelled.current = true
      clearInterval(interval)
    }
  }, [addr, active, dispatch])
}

const useDaemonStatus = (addr: string | undefined): UseDaemonStatusResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { pollFailed, status } = state

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  usePollStatus(addr, !pollFailed, dispatch)

  return { pollFailed, resumePolling, status }
}

export default useDaemonStatus
