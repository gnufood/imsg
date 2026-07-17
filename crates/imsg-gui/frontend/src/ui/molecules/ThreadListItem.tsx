import Avatar from '@/ui/atoms/Avatar.tsx'
import Badge from '@/ui/atoms/Badge.tsx'
import { MessageSquareDot } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'
import type { ThreadDto } from '@/bindings.ts'
import { useCallback } from 'react'

interface ThreadListItemProps {
  thread: ThreadDto
  onSelect: (address: string) => void
}

// `latest_ms` is a wall-clock instant, not a duration — `toLocaleString` (no added date lib)
// Is enough for a thread-row timestamp.
const formatLatest = (latestMs: bigint): string => new Date(Number(latestMs)).toLocaleString()

// Expects a `<ul>`/`<ol>` ancestor — ThreadList owns the list semantics, this is just the `<li>`.
const ThreadListItem = ({ thread, onSelect }: ThreadListItemProps): React.JSX.Element => {
  const handleClick = useCallback(() => {
    onSelect(thread.address)
  }, [thread.address, onSelect])

  return (
    <li>
      <button
        type="button"
        className="flex w-full items-center gap-3 rounded-md border border-line px-3.5 py-2.5 text-left text-sm text-ink hover:bg-ink/5"
        onClick={handleClick}
      >
        <Avatar />
        <span className="flex min-w-0 flex-1 flex-col items-start">
          <Text as="span">{thread.address}</Text>
          <Text as="span" size="xs" tone="muted">
            {formatLatest(thread.latest_ms)}
          </Text>
        </span>
        {thread.unread > 0n && <Badge count={Number(thread.unread)} icon={MessageSquareDot} />}
      </button>
    </li>
  )
}

export default ThreadListItem
