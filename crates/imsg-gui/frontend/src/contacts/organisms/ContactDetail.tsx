import Avatar from '@/ui/atoms/Avatar.tsx'
import type { ContactDto } from '@/bindings.ts'
import EmptyState from '@/ui/molecules/EmptyState.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import Text from '@/ui/atoms/Text.tsx'

const renderPhones = (phones: string[]): React.JSX.Element => {
  if (phones.length === 0) {
    return (
      <Text size="xs" tone="muted">
        No phone numbers on file.
      </Text>
    )
  }
  return (
    <ul className="flex flex-col gap-1">
      {phones.map((phone) => (
        <li key={phone}>
          <Text as="span">{phone}</Text>
        </li>
      ))}
    </ul>
  )
}

interface ContactDetailProps {
  contact: ContactDto | undefined
  // Caller-supplied so the same organism serves both the browse-selection empty state ("Select a
  // Contact.") and the search-result not-found state ("No contact found for …") — this component
  // Stays ignorant of which mode produced an absent contact, per the atomic-design spec.
  emptyMessage: string
  failed: boolean
  loading: boolean
  onRetry: () => void
}

const ContactDetail = ({ contact, emptyMessage, failed, loading, onRetry }: ContactDetailProps): React.JSX.Element => {
  if (loading) {
    return <LoadingState message="Loading contact…" />
  }
  if (failed) {
    return <ErrorState message="Couldn't load this contact." onRetry={onRetry} />
  }
  if (contact === undefined) {
    return <EmptyState message={emptyMessage} />
  }

  return (
    <div className="flex flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <Avatar />
        <Text as="h2">{contact.display_name ?? contact.uid}</Text>
      </div>
      {renderPhones(contact.phones)}
    </div>
  )
}

export default ContactDetail
