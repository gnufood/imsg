import { useCallback, useEffect, useReducer } from 'react'
import type UseContactsListResult from '@/contacts/application/use-contacts-list.types.ts'
import { commands } from '@/bindings.ts'

const PAGE_SIZE = 20

interface State {
  contacts: UseContactsListResult['contacts']
  failed: boolean
  page: number
  retryToken: number
}

const initialState: State = { contacts: undefined, failed: false, page: 0, retryToken: 0 }

type Action =
  | { contacts: NonNullable<UseContactsListResult['contacts']>; type: 'contactsReceived' }
  | { type: 'loadFailed' }
  | { type: 'nextPage' }
  | { type: 'prevPage' }
  | { type: 'retry' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'contactsReceived': {
      return { ...state, contacts: action.contacts, failed: false }
    }
    case 'loadFailed': {
      return { ...state, failed: true }
    }
    case 'nextPage': {
      return { ...state, page: state.page + 1 }
    }
    case 'prevPage': {
      return { ...state, page: Math.max(0, state.page - 1) }
    }
    case 'retry': {
      return { ...state, failed: false, retryToken: state.retryToken + 1 }
    }
  }
}

interface LoadArgs {
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
  page: number
}

const loadPage = async ({ cancelled, dispatch, page }: LoadArgs): Promise<void> => {
  const result = await commands.listContacts(PAGE_SIZE, page * PAGE_SIZE)
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ contacts: result.data, type: 'contactsReceived' })
    return
  }
  dispatch({ type: 'loadFailed' })
}

const usePageLoad = (page: number, retryToken: number, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    const cancelled = { current: false }
    void loadPage({ cancelled, dispatch, page })
    return () => {
      cancelled.current = true
    }
  }, [page, retryToken, dispatch])
}

// Application boundary for the contacts feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `listContacts`. `hasNextPage` is a heuristic
// (a full page came back), not a real total count — `list_contacts` doesn't return one.
const useContactsList = (): UseContactsListResult => {
  const [state, dispatch] = useReducer(reduce, initialState)
  const { contacts, failed, page, retryToken } = state

  usePageLoad(page, retryToken, dispatch)

  const nextPage = useCallback(() => {
    dispatch({ type: 'nextPage' })
  }, [])

  const prevPage = useCallback(() => {
    dispatch({ type: 'prevPage' })
  }, [])

  const retry = useCallback(() => {
    dispatch({ type: 'retry' })
  }, [])

  return {
    contacts,
    failed,
    hasNextPage: contacts !== undefined && contacts.length === PAGE_SIZE,
    hasPrevPage: page > 0,
    nextPage,
    prevPage,
    retry,
  }
}

export default useContactsList
