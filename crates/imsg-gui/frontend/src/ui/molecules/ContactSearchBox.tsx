import { useCallback, useState } from 'react'
import Button from '@/ui/atoms/Button.tsx'
import CompactButton from '@/ui/atoms/CompactButton.tsx'
import { Search } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'
import TextInput from '@/ui/atoms/TextInput.tsx'

interface ContactSearchBoxProps {
  active: boolean
  error: string | undefined
  onClear: () => void
  onSearch: (number: string) => void
  searching: boolean
}

// `active` (a search is currently showing a result) is distinct from `draft` having text — the
// User can type without having submitted yet, and "Clear search" needs to be offered based on
// Whether a search actually ran, not on draft content.
const ContactSearchBox = ({ active, error, onClear, onSearch, searching }: ContactSearchBoxProps): React.JSX.Element => {
  const [draft, setDraft] = useState('')

  const trimmed = draft.trim()
  const canSearch = trimmed.length > 0 && !searching

  const handleSearch = useCallback(() => {
    if (trimmed.length === 0 || searching) {
      return
    }
    onSearch(trimmed)
  }, [trimmed, searching, onSearch])

  const handleClear = useCallback(() => {
    setDraft('')
    onClear()
  }, [onClear])

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        handleSearch()
      }
    },
    [handleSearch],
  )

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <TextInput value={draft} onChange={setDraft} onKeyDown={handleKeyDown} placeholder="Search by phone number" disabled={searching} />
        <Button onClick={handleSearch} icon={Search} disabled={!canSearch}>
          Search
        </Button>
      </div>
      {error !== undefined && (
        <Text size="xs" tone="muted">
          {error}
        </Text>
      )}
      {active && <CompactButton onClick={handleClear}>Clear search</CompactButton>}
    </div>
  )
}

export default ContactSearchBox
