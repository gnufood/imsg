import { useCallback, useState } from 'react'
import type { ContactDto } from '@/bindings.ts'
import Contacts from '@/contacts/pages/Contacts.tsx'
import useContactDetail from '@/contacts/application/use-contact-detail.ts'
import useContactLookup from '@/contacts/application/use-contact-lookup.ts'
import useContactsList from '@/contacts/application/use-contacts-list.ts'
import useContactsSync from '@/contacts/application/use-contacts-sync.ts'

interface ResolvedDetailArgs {
  searchActive: boolean
  searchContact: ContactDto | null | undefined
  searching: boolean
  selectedContact: ContactDto | undefined
  selectedFailed: boolean
  selectedUid: string | undefined
}

interface ResolvedDetail {
  contact: ContactDto | undefined
  emptyMessage: string
  failed: boolean
  loading: boolean
}

// A search result, while active, always wins over whatever's list-selected — see
// `ContactSearchBox`'s own doc comment on `active` for why the two are kept separate rather than
// Merged into one piece of state.
const resolveDetail = ({ searchActive, searchContact, searching, selectedContact, selectedFailed, selectedUid }: ResolvedDetailArgs): ResolvedDetail => {
  if (searchActive) {
    return { contact: searchContact ?? undefined, emptyMessage: 'No contact found for that number.', failed: false, loading: searching }
  }
  return {
    contact: selectedContact,
    emptyMessage: 'Select a contact from the list.',
    failed: selectedFailed,
    loading: selectedUid !== undefined && selectedContact === undefined && !selectedFailed,
  }
}

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useContactsList`/`useContactDetail`/`useContactLookup`/`useContactsSync`'s real backend calls
// And `Contacts`'s presentational page. Owns `selectedUid` itself, the one piece of state shared
// Across the list/detail hooks (mirrors `MessagesConnected`'s `selectedAddress`). Its own
// `useContactsSync` instance is independent from `MessagesConnected`'s (see that hook's doc
// Comment) — this one backs the page's own refresh button, not `ThreadListPane`'s.
const ContactsConnected = (): React.JSX.Element => {
  const { contacts, failed: listFailed, hasNextPage, hasPrevPage, nextPage, prevPage, retry: retryList } = useContactsList()
  const [selectedUid, setSelectedUid] = useState<string | undefined>()
  const { contact: selectedContact, failed: selectedFailed, retry: retryDetail } = useContactDetail(selectedUid)
  const { clear: clearSearch, contact: searchContact, error: searchError, search, searching } = useContactLookup()
  const { error: syncError, sync: refresh, syncing: refreshing } = useContactsSync()

  const searchActive = searching || searchContact !== undefined
  const detail = resolveDetail({ searchActive, searchContact, searching, selectedContact, selectedFailed, selectedUid })

  const handleSelectContact = useCallback(
    (uid: string) => {
      clearSearch()
      setSelectedUid(uid)
    },
    [clearSearch],
  )

  return (
    <Contacts
      contacts={contacts}
      detailContact={detail.contact}
      detailEmptyMessage={detail.emptyMessage}
      detailFailed={detail.failed}
      detailLoading={detail.loading}
      hasNextPage={hasNextPage}
      hasPrevPage={hasPrevPage}
      listFailed={listFailed}
      onClearSearch={clearSearch}
      onNextPage={nextPage}
      onPrevPage={prevPage}
      onRefresh={refresh}
      onRetryDetail={retryDetail}
      onRetryList={retryList}
      onSearch={search}
      onSelectContact={handleSelectContact}
      refreshing={refreshing}
      searchActive={searchActive}
      searchError={searchError}
      searching={searching}
      syncError={syncError}
    />
  )
}

export default ContactsConnected
