import type { ContactDto, ContactEntryDto } from '@/bindings.ts'
import ContactsTemplate from '@/contacts/templates/ContactsTemplate.tsx'

interface ContactsProps {
  contacts: ContactEntryDto[] | undefined
  detailContact: ContactDto | undefined
  detailEmptyMessage: string
  detailFailed: boolean
  detailLoading: boolean
  hasNextPage: boolean
  hasPrevPage: boolean
  listFailed: boolean
  onClearSearch: () => void
  onNextPage: () => void
  onPrevPage: () => void
  onRefresh: () => void
  onRetryDetail: () => void
  onRetryList: () => void
  onSearch: (number: string) => void
  onSelectContact: (uid: string) => void
  refreshing: boolean
  searchActive: boolean
  searchError: string | undefined
  searching: boolean
  syncError: string | undefined
}

// Pure forwarder — all state lives in `ContactsConnected`'s hooks, same shape as `Settings.tsx`.
const Contacts = ({
  contacts,
  detailContact,
  detailEmptyMessage,
  detailFailed,
  detailLoading,
  hasNextPage,
  hasPrevPage,
  listFailed,
  onClearSearch,
  onNextPage,
  onPrevPage,
  onRefresh,
  onRetryDetail,
  onRetryList,
  onSearch,
  onSelectContact,
  refreshing,
  searchActive,
  searchError,
  searching,
  syncError,
}: ContactsProps): React.JSX.Element => (
  <ContactsTemplate
    contacts={contacts}
    detailContact={detailContact}
    detailEmptyMessage={detailEmptyMessage}
    detailFailed={detailFailed}
    detailLoading={detailLoading}
    hasNextPage={hasNextPage}
    hasPrevPage={hasPrevPage}
    listFailed={listFailed}
    onClearSearch={onClearSearch}
    onNextPage={onNextPage}
    onPrevPage={onPrevPage}
    onRefresh={onRefresh}
    onRetryDetail={onRetryDetail}
    onRetryList={onRetryList}
    onSearch={onSearch}
    onSelectContact={onSelectContact}
    refreshing={refreshing}
    searchActive={searchActive}
    searchError={searchError}
    searching={searching}
    syncError={syncError}
  />
)

export default Contacts
