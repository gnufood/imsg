import { useCallback, useState } from 'react'
import Button from '@/ui/atoms/Button.tsx'
import { Send } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'
import TextInput from '@/ui/atoms/TextInput.tsx'

interface MessageComposerProps {
  error: string | undefined
  onSend: (text: string) => void
  pending: boolean
}

const MessageComposer = ({ error, onSend, pending }: MessageComposerProps): React.JSX.Element => {
  const [draft, setDraft] = useState('')

  const trimmed = draft.trim()
  const canSend = trimmed.length > 0 && !pending

  const handleSend = useCallback(() => {
    if (trimmed.length === 0 || pending) {
      return
    }
    onSend(trimmed)
    setDraft('')
  }, [trimmed, pending, onSend])

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        handleSend()
      }
    },
    [handleSend],
  )

  return (
    <div className="flex flex-col gap-2 border-t border-line p-3">
      {error !== undefined && (
        <Text size="xs" tone="muted">
          {error}
        </Text>
      )}
      <div className="flex items-center gap-2">
        <TextInput value={draft} onChange={setDraft} onKeyDown={handleKeyDown} placeholder="Message" disabled={pending} />
        <Button onClick={handleSend} icon={Send} disabled={!canSend}>
          Send
        </Button>
      </div>
    </div>
  )
}

export default MessageComposer
