import { useCallback, useEffect, useReducer } from 'react'
import type UseThreadsResult from '@/messages/application/use-threads.types.ts'
import { commands } from '@/bindings.ts'

// 5s, not the gate's 250ms — this isn't gating startup, just keeping the list current (see
// GUI_NEXT.md "Live updates — decided: poll").
const POLL_MS = 5000

interface State {
  pollFailed: boolean
  threads: UseThreadsResult['threads']
}

const initialState: State = { pollFailed: false, threads: undefined }

type Action = { threads: NonNullable<UseThreadsResult['threads']>; type: 'threadsReceived' } | { type: 'pollFailed' } | { type: 'resumePolling' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'threadsReceived': {
      return { ...state, pollFailed: false, threads: action.threads }
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

const pollThreads = async ({ cancelled, dispatch }: PollArgs): Promise<void> => {
  const result = await commands.threads()
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ threads: result.data, type: 'threadsReceived' })
    return
  }
  dispatch({ type: 'pollFailed' })
}

const usePollThreads = (active: boolean, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (!active) {
      return
    }
    const cancelled = { current: false }
    void pollThreads({ cancelled, dispatch })
    const interval = setInterval(() => {
      void pollThreads({ cancelled, dispatch })
    }, POLL_MS)
    return () => {
      cancelled.current = true
      clearInterval(interval)
    }
  }, [active, dispatch])
}

// Application boundary for the messages feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`.
const useThreads = (): UseThreadsResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { pollFailed, threads } = state

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  usePollThreads(!pollFailed, dispatch)

  return { pollFailed, resumePolling, threads }
}

export default useThreads
