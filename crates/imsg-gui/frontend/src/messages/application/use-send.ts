import { useCallback, useReducer } from 'react'
import type UseSendResult from '@/messages/application/use-send.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  error: string | undefined
  sending: boolean
}

const initialState: State = { error: undefined, sending: false }

type Action = { type: 'sendStarted' } | { type: 'sendSucceeded' } | { message: string; type: 'sendFailed' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'sendStarted': {
      return { ...state, error: undefined, sending: true }
    }
    case 'sendSucceeded': {
      return { ...state, error: undefined, sending: false }
    }
    case 'sendFailed': {
      return { ...state, error: action.message, sending: false }
    }
  }
}

interface SendArgs {
  dispatch: React.Dispatch<Action>
  recipient: string
  text: string
}

// Looks up the device's own address fresh on every send rather than caching it — sends are
// Infrequent, and this avoids a second piece of "is it loaded yet" state.
const sendMessage = async ({ dispatch, recipient, text }: SendArgs): Promise<void> => {
  dispatch({ type: 'sendStarted' })
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const config = await commands.configShow(null)
  if (config.status === 'error') {
    dispatch({ message: config.error.message, type: 'sendFailed' })
    return
  }
  const result = await commands.send(config.data.device_address, recipient, text)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'sendFailed' })
    return
  }
  dispatch({ type: 'sendSucceeded' })
}

// Application boundary for the messages feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here (alongside use-threads.ts/use-conversation.ts) allowed to import `bindings.ts`.
// `recipient` is the selected thread's address; `undefined` when no thread is selected, in which
// Case `send` is a no-op.
const useSend = (recipient: string | undefined): UseSendResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const send = useCallback(
    (text: string) => {
      if (recipient === undefined) {
        return
      }
      void sendMessage({ dispatch, recipient, text })
    },
    [recipient],
  )

  return { ...state, send }
}

export default useSend
