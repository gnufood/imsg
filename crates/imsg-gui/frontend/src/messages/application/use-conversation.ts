import { useCallback, useEffect, useReducer } from 'react'
import type UseConversationResult from '@/messages/application/use-conversation.types.ts'
import { commands } from '@/bindings.ts'

// Same 5s cadence as use-threads.ts (see GUI_NEXT.md "Live updates — decided: poll").
const POLL_MS = 5000
// No pagination yet — a flat cap keeps the first read simple; revisit if real conversations
// Exceed it.
const MESSAGE_LIMIT = 200

interface State {
  messages: UseConversationResult['messages']
  pollFailed: boolean
}

const initialState: State = { messages: undefined, pollFailed: false }

type Action =
  | { messages: NonNullable<UseConversationResult['messages']>; type: 'messagesReceived' }
  | { type: 'pollFailed' }
  | { type: 'resumePolling' }
  | { type: 'addressChanged' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'messagesReceived': {
      return { ...state, messages: action.messages, pollFailed: false }
    }
    case 'pollFailed': {
      return { ...state, pollFailed: true }
    }
    case 'resumePolling': {
      return { ...state, pollFailed: false }
    }
    case 'addressChanged': {
      return { ...state, messages: undefined }
    }
  }
}

interface PollArgs {
  address: string
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
}

// `listMessages` returns newest-first (see bindings.ts) — reversed here so the conversation
// Renders oldest-at-top, chat convention.
const pollMessages = async ({ address, cancelled, dispatch }: PollArgs): Promise<void> => {
  const result = await commands.listMessages(null, false, address, null, MESSAGE_LIMIT, 0)
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ messages: result.data.toReversed(), type: 'messagesReceived' })
    return
  }
  dispatch({ type: 'pollFailed' })
}

// Fires only when the selected address itself changes, not on every `active`/`pollFailed`
// Toggle — otherwise a retry-after-failure would flash the view back to "loading" and lose
// The last-known messages for no reason.
const useResetOnAddressChange = (address: string | undefined, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    dispatch({ type: 'addressChanged' })
  }, [address, dispatch])
}

const usePollMessages = (address: string | undefined, active: boolean, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (address === undefined || !active) {
      return
    }
    const cancelled = { current: false }
    void pollMessages({ address, cancelled, dispatch })
    const interval = setInterval(() => {
      void pollMessages({ address, cancelled, dispatch })
    }, POLL_MS)
    return () => {
      cancelled.current = true
      clearInterval(interval)
    }
  }, [address, active, dispatch])
}

// Application boundary for the messages feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here (alongside use-threads.ts) allowed to import `bindings.ts`. `address` is
// `undefined` when no thread is selected — polling simply doesn't start.
const useConversation = (address: string | undefined): UseConversationResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { messages, pollFailed } = state

  const resumePolling = useCallback(() => {
    dispatch({ type: 'resumePolling' })
  }, [])

  useResetOnAddressChange(address, dispatch)
  usePollMessages(address, !pollFailed, dispatch)

  return { messages, pollFailed, resumePolling }
}

export default useConversation
