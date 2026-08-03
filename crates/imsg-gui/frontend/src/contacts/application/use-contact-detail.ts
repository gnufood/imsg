import { useCallback, useEffect, useReducer } from 'react'
import type UseContactDetailResult from '@/contacts/application/use-contact-detail.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  contact: UseContactDetailResult['contact']
  failed: boolean
  retryToken: number
}

const initialState: State = { contact: undefined, failed: false, retryToken: 0 }

type Action =
  | { contact: NonNullable<UseContactDetailResult['contact']> | undefined; type: 'contactReceived' }
  | { type: 'loadFailed' }
  | { type: 'uidChanged' }
  | { type: 'retry' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'contactReceived': {
      return { ...state, contact: action.contact, failed: false }
    }
    case 'loadFailed': {
      return { ...state, failed: true }
    }
    case 'uidChanged': {
      return { ...state, contact: undefined, failed: false }
    }
    case 'retry': {
      return { ...state, failed: false, retryToken: state.retryToken + 1 }
    }
  }
}

interface LoadArgs {
  cancelled: { current: boolean }
  dispatch: React.Dispatch<Action>
  uid: string
}

const loadContact = async ({ cancelled, dispatch, uid }: LoadArgs): Promise<void> => {
  const result = await commands.getContact(uid)
  if (cancelled.current) {
    return
  }
  if (result.status === 'ok') {
    dispatch({ contact: result.data ?? undefined, type: 'contactReceived' })
    return
  }
  dispatch({ type: 'loadFailed' })
}

// `retryToken` deliberately isn't in the reset branch below — resetting to `undefined` on every
// Retry would flash the view back to "loading" for no reason; `loadContact` overwrites it once
// The retried fetch resolves either way.
const useLoadOnUidChange = (uid: string | undefined, retryToken: number, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    if (uid === undefined) {
      return
    }
    const cancelled = { current: false }
    void loadContact({ cancelled, dispatch, uid })
    return () => {
      cancelled.current = true
    }
  }, [uid, retryToken, dispatch])
}

const useResetOnUidChange = (uid: string | undefined, dispatch: React.Dispatch<Action>): void => {
  useEffect(() => {
    dispatch({ type: 'uidChanged' })
  }, [uid, dispatch])
}

// Application boundary for the contacts feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `getContact`. `uid` is `undefined` when no
// List row is selected — fetching simply doesn't start. One-shot per `uid` (plus `retry`), not
// Polled — unlike the MAP-backed conversation view, PBAP has no push channel, so a selected
// Contact only changes via an explicit sync (see `use-contacts-sync.ts`), not in the background.
const useContactDetail = (uid: string | undefined): UseContactDetailResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  useResetOnUidChange(uid, dispatch)
  useLoadOnUidChange(uid, state.retryToken, dispatch)

  const retry = useCallback(() => {
    dispatch({ type: 'retry' })
  }, [])

  return { ...state, retry }
}

export default useContactDetail
