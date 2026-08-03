import Avatar from '@/ui/atoms/Avatar.tsx'
import type { ContactEntryDto } from '@/bindings.ts'
import Text from '@/ui/atoms/Text.tsx'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface ContactListItemProps {
  contact: ContactEntryDto
  onSelect: (uid: string) => void
}

// Expects a `<ul>`/`<ol>` ancestor — ContactList owns the list semantics, this is just the `<li>`.
const ContactListItem = ({ contact, onSelect }: ContactListItemProps): React.JSX.Element => {
  const handleClick = useCallback(() => {
    onSelect(contact.uid)
  }, [contact.uid, onSelect])

  return (
    <li>
      <button
        type="button"
        className={`flex w-full items-center gap-3 rounded-md border border-line px-3.5 py-2.5 text-left text-sm text-ink ${interactiveStyles.hover} ${interactiveStyles.focus}`}
        onClick={handleClick}
      >
        <Avatar />
        <Text as="span">{contact.display_name ?? contact.uid}</Text>
      </button>
    </li>
  )
}

export default ContactListItem
