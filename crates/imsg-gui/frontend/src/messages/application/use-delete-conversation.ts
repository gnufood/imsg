import { useCallback, useReducer } from 'react'
import type UseConversationResult from '@/messages/application/use-conversation.types.ts'
import type UseDeleteConversationResult from '@/messages/application/use-delete-conversation.types.ts'
import { commands } from '@/bindings.ts'

// Reuses `use-conversation.ts`'s message type rather than importing `MessageDto` from
// `bindings.ts` directly — keeps this file's only `bindings.ts` import to `commands`, matching
// `use-mark-read.ts`'s identical pattern.
type Messages = NonNullable<UseConversationResult['messages']>

interface State {
  confirmOpen: boolean
  deleting: boolean
  error: string | undefined
}

const initialState: State = { confirmOpen: false, deleting: false, error: undefined }

type Action =
  | { type: 'requestDelete' }
  | { type: 'cancelDelete' }
  | { type: 'deleteStarted' }
  | { type: 'deleteSucceeded' }
  | { message: string; type: 'deleteFailed' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'requestDelete': {
      return { ...state, confirmOpen: true, error: undefined }
    }
    case 'cancelDelete': {
      return { ...state, confirmOpen: false }
    }
    case 'deleteStarted': {
      return { ...state, deleting: true }
    }
    case 'deleteSucceeded': {
      return { ...state, confirmOpen: false, deleting: false }
    }
    case 'deleteFailed': {
      // Dialog stays open (unlike the success case) — the failure is shown inside it, and the
      // User needs Cancel or a retried Confirm, not to be dropped back to the conversation.
      return { ...state, deleting: false, error: action.message }
    }
  }
}

interface DeleteAllArgs {
  addr: string
  dispatch: React.Dispatch<Action>
  messages: Messages
  onDeleted: () => void
}

// Best-effort: fires every delete concurrently and surfaces one error if any failed, rather than
// Aborting the batch on the first failure — the deletes that did succeed still need to stick.
const deleteAll = async ({ addr, dispatch, messages, onDeleted }: DeleteAllArgs): Promise<void> => {
  dispatch({ type: 'deleteStarted' })
  const results = await Promise.all(messages.map((message) => commands.delete(addr, message.handle, message.folder)))
  const failed = results.find((result) => result.status === 'error')
  if (failed !== undefined && failed.status === 'error') {
    dispatch({ message: failed.error.message, type: 'deleteFailed' })
    return
  }
  dispatch({ type: 'deleteSucceeded' })
  onDeleted()
}

interface ConfirmDeleteArgs {
  dispatch: React.Dispatch<Action>
  messages: Messages
  onDeleted: () => void
}

const confirmDeleteConversation = async ({ dispatch, messages, onDeleted }: ConfirmDeleteArgs): Promise<void> => {
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const config = await commands.configShow(null)
  if (config.status === 'error') {
    dispatch({ message: config.error.message, type: 'deleteFailed' })
    return
  }
  void deleteAll({ addr: config.data.device_address, dispatch, messages, onDeleted })
}

// Application boundary for the messages feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here (alongside use-threads.ts/use-conversation.ts/use-send.ts) allowed to import
// `bindings.ts`'s `commands`. `messages` is the already-polled conversation (same MESSAGE_LIMIT
// Cap as use-conversation.ts) — deleting doesn't re-fetch a fresher or unbounded list.
const useDeleteConversation = (
  address: string | undefined,
  messages: Messages | undefined,
  onDeleted: () => void,
): UseDeleteConversationResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const requestDelete = useCallback(() => {
    dispatch({ type: 'requestDelete' })
  }, [])

  const cancelDelete = useCallback(() => {
    dispatch({ type: 'cancelDelete' })
  }, [])

  const confirmDelete = useCallback(() => {
    if (address === undefined || messages === undefined) {
      return
    }
    void confirmDeleteConversation({ dispatch, messages, onDeleted })
  }, [address, messages, onDeleted])

  return { ...state, cancelDelete, confirmDelete, requestDelete }
}

export default useDeleteConversation
