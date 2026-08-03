import { useCallback, useReducer } from 'react'
import type UseContactsSyncResult from '@/contacts/application/use-contacts-sync.types.ts'
import { commands } from '@/bindings.ts'

type Report = UseContactsSyncResult['report']

interface State {
  error: string | undefined
  report: Report
  syncing: boolean
}

const initialState: State = { error: undefined, report: undefined, syncing: false }

type Action = { type: 'syncStarted' } | { report: NonNullable<Report>; type: 'syncSucceeded' } | { message: string; type: 'syncFailed' }

const reduce = (state: State, action: Action): State => {
  switch (action.type) {
    case 'syncStarted': {
      return { ...state, error: undefined, syncing: true }
    }
    case 'syncSucceeded': {
      return { ...state, error: undefined, report: action.report, syncing: false }
    }
    case 'syncFailed': {
      return { ...state, error: action.message, syncing: false }
    }
  }
}

interface SyncArgs {
  dispatch: React.Dispatch<Action>
}

// Looks up the device's own address fresh on every sync rather than caching it — same reasoning
// As `use-send.ts`'s `sendMessage`, and avoids requiring the caller (`MessagesConnected`) to plumb
// The device address down just for this one action.
const syncContacts = async ({ dispatch }: SyncArgs): Promise<void> => {
  dispatch({ type: 'syncStarted' })
  // `configPath` is `string | null` (specta's mirror of Rust's `Option<T>`) — `null` here means
  // "use the default config file location," the only way to express that over this wire contract.
  // eslint-disable-next-line unicorn/no-null
  const config = await commands.configShow(null)
  if (config.status === 'error') {
    dispatch({ message: config.error.message, type: 'syncFailed' })
    return
  }
  const result = await commands.syncContactsNow(config.data.device_address)
  if (result.status === 'error') {
    dispatch({ message: result.error.message, type: 'syncFailed' })
    return
  }
  dispatch({ report: result.data, type: 'syncSucceeded' })
}

// Application boundary for the contacts sync action (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here allowed to import `bindings.ts`'s `syncContactsNow`. Backs `MessagesConnected`'s
// `ThreadListPane` header refresh button; the PBAP sync it triggers is what keeps `contact_name`
// Resolution current even without a dedicated browse-contacts page.
const useContactsSync = (): UseContactsSyncResult => {
  const [state, dispatch] = useReducer(reduce, initialState)

  const sync = useCallback(() => {
    void syncContacts({ dispatch })
  }, [])

  return { ...state, sync }
}

export default useContactsSync
