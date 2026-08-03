import { useCallback, useEffect, useReducer } from 'react'
import type UseGateStatusResult from '@/gate/application/use-gate-status.types.ts'
import { commands } from '@/bindings.ts'

const POLL_MS = 250

type Status = UseGateStatusResult['status']

interface State {
  pollFailed: boolean
  status: Status
}

const initialState: State = { pollFailed: false, status: undefined }

type Action = { status: NonNullable<Status>; type: 'statusReceived' } | { type: 'pollFailed' } | { type: 'resumePolling' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'statusReceived': {
      return { ...state, status: action.status }
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

const pollStatus = async ({ cancelled, dispatch }: PollArgs): Promise<void> => {
  try {
    const status = await commands.gateStatus()
    if (!cancelled.current) {
      dispatch({ status, type: 'statusReceived' })
    }
  } catch {
    if (!cancelled.current) {
      dispatch({ type: 'pollFailed' })
    }
  }
}

const requestProceed = async (dispatch: React.Dispatch<Action>): Promise<void> => {
  try {
    await commands.gateProceed()
  } catch {
    dispatch({ type: 'pollFailed' })
  }
}

const usePollStatus = (active: boolean, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (!active) {
      return
    }
    const cancelled = { current: false }
    void pollStatus({ cancelled, dispatch })
    const interval = setInterval(() => {
      void pollStatus({ cancelled, dispatch })
    }, POLL_MS)
    return () => {
      cancelled.current = true
      clearInterval(interval)
    }
  }, [active, dispatch])
}

const useGateStatus = (): UseGateStatusResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { pollFailed, status } = state

  const proceed = useCallback(() => {
    void requestProceed(dispatch)
  }, [])

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  usePollStatus(!pollFailed && status !== 'Ready', dispatch)

  return { pollFailed, proceed, resumePolling, status }
}

export default useGateStatus
