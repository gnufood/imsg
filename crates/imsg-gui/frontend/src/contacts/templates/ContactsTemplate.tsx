import type { ContactDto, ContactEntryDto } from '@/bindings.ts'
import CompactButton from '@/ui/atoms/CompactButton.tsx'
import ContactDetail from '@/contacts/organisms/ContactDetail.tsx'
import ContactList from '@/contacts/organisms/ContactList.tsx'
import ContactSearchBox from '@/ui/molecules/ContactSearchBox.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import IconButton from '@/ui/atoms/IconButton.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import { RefreshCw } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'

interface ListPaneArgs {
  contacts: ContactEntryDto[] | undefined
  hasNextPage: boolean
  hasPrevPage: boolean
  listFailed: boolean
  onNextPage: () => void
  onPrevPage: () => void
  onRetryList: () => void
  onSelectContact: (uid: string) => void
}

// Pulled out of `ContactsTemplate` so it stays under this repo's max-lines-per-function limit.
const renderListPane = ({ contacts, hasNextPage, hasPrevPage, listFailed, onNextPage, onPrevPage, onRetryList, onSelectContact }: ListPaneArgs): React.JSX.Element => {
  if (listFailed) {
    return <ErrorState message="Couldn't load contacts." onRetry={onRetryList} />
  }
  if (contacts === undefined) {
    return <LoadingState message="Loading contacts…" />
  }
  return (
    <div className="flex flex-col gap-3">
      <ContactList contacts={contacts} onSelect={onSelectContact} />
      <div className="flex items-center justify-between">
        <CompactButton disabled={!hasPrevPage} onClick={onPrevPage}>
          Previous
        </CompactButton>
        <CompactButton disabled={!hasNextPage} onClick={onNextPage}>
          Next
        </CompactButton>
      </div>
    </div>
  )
}

interface ContactsTemplateProps {
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

const ContactsTemplate = ({
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
}: ContactsTemplateProps): React.JSX.Element => (
  <div className="flex h-screen w-full flex-col bg-surface text-ink">
    <div className="flex items-center justify-between border-b border-line p-4">
      <Text as="h1">Contacts</Text>
      <div className="flex items-center gap-2">
        {syncError !== undefined && (
          <Text size="xs" tone="muted">
            {syncError}
          </Text>
        )}
        <IconButton disabled={refreshing} icon={RefreshCw} label="Refresh contacts" onClick={onRefresh} />
      </div>
    </div>
    <div className="flex min-h-0 flex-1">
      <div className="flex w-72 shrink-0 flex-col gap-3 overflow-y-auto border-r border-line p-4">
        <ContactSearchBox active={searchActive} error={searchError} onClear={onClearSearch} onSearch={onSearch} searching={searching} />
        {renderListPane({ contacts, hasNextPage, hasPrevPage, listFailed, onNextPage, onPrevPage, onRetryList, onSelectContact })}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <ContactDetail contact={detailContact} emptyMessage={detailEmptyMessage} failed={detailFailed} loading={detailLoading} onRetry={onRetryDetail} />
      </div>
    </div>
  </div>
)

export default ContactsTemplate
