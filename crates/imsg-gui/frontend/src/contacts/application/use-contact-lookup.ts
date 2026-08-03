import { useCallback, useReducer } from 'react'
import type UseContactLookupResult from '@/contacts/application/use-contact-lookup.types.ts'
import { commands } from '@/bindings.ts'

interface State {
  contact: UseContactLookupResult['contact']
  error: string | undefined
  searching: boolean
}

const initialState: State = { contact: undefined, error: undefined, searching: false }

type Action = { type: 'searchStarted' } | { contact: NonNullable<UseContactLookupResult['contact']> | null; type: 'searchSucceeded' } | { message: string; type: 'searchFailed' } | { type: 'cleared' }

// Pure — every transition names the state it lands on explicitly.
const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'searchStarted': {
      return { ...state, error: undefined, searching: true }
    }
    case 'searchSucceeded': {
      return { ...state, contact: action.contact, error: undefined, searching: false }
    }
    case 'searchFailed': {
      return { ...state, error: action.message, searching: false }
    }
    case 'cleared': {
      return initialState
    }
  }
}

interface SearchArgs {
  dispatch: React.Dispatch<Action>
  number: string
}

const searchByNumber = async ({ dispatch, number }: SearchArgs): Promise<void> => {
  dispatch({ type: 'searchStarted' })
  const result = await commands.lookupContact(number)
  if (result.status === 'ok') {
    dispatch({ contact: result.data, type: 'searchSucceeded' })
    return
  }
  dispatch({ message: result.error.message, type: 'searchFailed' })
}

// Application boundary for the contacts feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `lookupContact`. Explicit `search`/`clear`
// Actions, not effect-driven — a search only runs when the user submits the search box, mirroring
// `use-send.ts`'s explicit-trigger shape rather than `use-contact-detail.ts`'s id-keyed effect.
const useContactLookup = (): UseContactLookupResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const search = useCallback((number: string) => {
    void searchByNumber({ dispatch, number })
  }, [])

  const clear = useCallback(() => {
    dispatch({ type: 'cleared' })
  }, [])

  return { ...state, clear, search }
}

export default useContactLookup
