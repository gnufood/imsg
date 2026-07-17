import { useCallback, useEffect, useReducer } from 'react'
import type UseGateStatusResult from '@/gate/application/use-gate-status.types.ts'
import { commands } from '@/bindings.ts'

// Backend owns all sequencing (`gate::run` in Rust, spawned once by main.rs). This hook only
// Mirrors its status and can poke it to re-evaluate — polling is the decided mechanism
// (authoritative at any time, no listener-mount race; see GUI.md).
const POLL_MS = 250

type Status = UseGateStatusResult['status']

interface State {
  pollFailed: boolean
  status: Status
}

const initialState: State = { pollFailed: false, status: undefined }

type Action = { status: NonNullable<Status>; type: 'statusReceived' } | { type: 'pollFailed' } | { type: 'resumePolling' }

// Pure — every transition names the state it lands on explicitly.
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

// `gateStatus` isn't `typedError`-wrapped (see bindings.ts) — a rejection is a genuine IPC
// Failure, not a modeled `CommandError`, so try/catch rather than a `result.status` branch.
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

// Application boundary for the gate feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`. Converts the polled backend status into the
// State/actions `Gate` (the page) renders.
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
