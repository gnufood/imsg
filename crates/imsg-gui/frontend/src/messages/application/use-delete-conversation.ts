import { useCallback, useReducer } from 'react'
import type UseConversationResult from '@/messages/application/use-conversation.types.ts'
import type UseDeleteConversationResult from '@/messages/application/use-delete-conversation.types.ts'
import { commands } from '@/bindings.ts'

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
  // eslint-disable-next-line unicorn/no-null
  const config = await commands.configShow(null)
  if (config.status === 'error') {
    dispatch({ message: config.error.message, type: 'deleteFailed' })
    return
  }
  void deleteAll({ addr: config.data.device_address, dispatch, messages, onDeleted })
}

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
