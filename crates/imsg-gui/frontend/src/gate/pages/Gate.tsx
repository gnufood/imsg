import { useCallback, useEffect, useReducer } from 'react'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import DeviceSetup from '@/gate/pages/DeviceSetup.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import Preparing from '@/gate/organisms/Preparing.tsx'
import Splash from '@/gate/organisms/Splash.tsx'
import { commands } from '@/bindings.ts'

// Derived from the command's own return type rather than a second `import type` from
// Bindings.ts — same convention as DeviceSetup's `PairedDeviceDto`.
type GateStatus = Awaited<ReturnType<typeof commands.gateStatus>>
type PreparingStage = Parameters<typeof Preparing>[0]['stage']

// The backend owns all sequencing (`gate::run` in Rust, spawned once by main.rs). This page
// Only mirrors its status and can poke it to re-evaluate — polling is the decided mechanism
// (authoritative at any time, no listener-mount race; see GUI.md).
const POLL_MS = 250

interface State {
  pollFailed: boolean
  splashDone: boolean
  status: GateStatus | undefined
}

const initialState: State = { pollFailed: false, splashDone: false, status: undefined }

type Action =
  | { status: GateStatus; type: 'statusReceived' }
  | { type: 'pollFailed' }
  | { type: 'resumePolling' }
  | { type: 'splashDone' }

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
    case 'splashDone': {
      return { ...state, splashDone: true }
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

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

const failedStage = (stage: 'Daemon' | 'Sync'): PreparingStage => {
  if (stage === 'Daemon') {
    return 'daemon'
  }
  return 'sync'
}

const renderStatus = (status: GateStatus | undefined, proceed: () => void): React.JSX.Element => {
  if (status === 'AwaitingDeviceConfig') {
    return <DeviceSetup onComplete={proceed} />
  }
  if (status === 'StartingDaemon') {
    return <Preparing stage="daemon" onRetry={proceed} />
  }
  if (status === 'Syncing') {
    return <Preparing stage="sync" onRetry={proceed} />
  }
  if (status !== undefined && status !== 'Initializing' && status !== 'Ready') {
    return <Preparing stage={failedStage(status.Failed.stage)} error={status.Failed.message} onRetry={proceed} />
  }
  // `undefined` (first poll in flight), 'Initializing', or 'Ready' — the last is a single
  // Transitional frame (the handoff Effect fires on this same render pass, then the parent
  // Swaps Gate out), reusing the loading treatment avoids rendering "nothing" (forbidden by
  // `unicorn/no-null`/`react/jsx-no-useless-fragment`).
  return screen(<LoadingState message="Starting up…" />)
}

interface RenderGateArgs {
  handleSplashDone: () => void
  proceed: () => void
  resumePolling: () => void
  state: State
}

const renderGate = ({ handleSplashDone, proceed, resumePolling, state }: RenderGateArgs): React.JSX.Element => {
  const { pollFailed, splashDone, status } = state
  if (!splashDone) {
    return <Splash ready={status !== undefined || pollFailed} onDone={handleSplashDone} />
  }
  if (pollFailed) {
    return screen(<ErrorState message="Couldn't reach the app backend." onRetry={resumePolling} />)
  }
  return renderStatus(status, proceed)
}

interface GateProps {
  // Caller-owned hand-off once the backend gate reports Ready — Gate only decides *when*,
  // Never what happens after.
  onReady: () => void
}

const Gate = ({ onReady }: GateProps): React.JSX.Element => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { pollFailed, splashDone, status } = state

  const proceed = useCallback(() => {
    void requestProceed(dispatch)
  }, [])

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  const handleSplashDone = useCallback(() => {
    dispatch({ type: 'splashDone' })
  }, [])

  usePollStatus(!pollFailed && status !== 'Ready', dispatch)

  // `Splash` may finish (its cap outlasts, not outraces, the floor) before the gate does —
  // RenderGate's loading branch after `splashDone` covers that window.
  useEffect(() => {
    if (splashDone && status === 'Ready') {
      onReady()
    }
  }, [splashDone, status, onReady])

  return renderGate({ handleSplashDone, proceed, resumePolling, state })
}

export default Gate
