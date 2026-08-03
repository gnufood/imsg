import type { ContactEntryDto } from '@/bindings.ts'
import ContactListItem from '@/ui/molecules/ContactListItem.tsx'
import EmptyState from '@/ui/molecules/EmptyState.tsx'

interface ContactListProps {
  contacts: ContactEntryDto[]
  onSelect: (uid: string) => void
}

const ContactList = ({ contacts, onSelect }: ContactListProps): React.JSX.Element => {
  if (contacts.length === 0) {
    return <EmptyState message="No contacts found." />
  }

  return (
    <ul className="flex w-full flex-col gap-2">
      {contacts.map((contact) => (
        <ContactListItem key={contact.uid} contact={contact} onSelect={onSelect} />
      ))}
    </ul>
  )
}

export default ContactList
