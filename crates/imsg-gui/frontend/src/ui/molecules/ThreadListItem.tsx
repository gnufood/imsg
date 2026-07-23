import Avatar from '@/ui/atoms/Avatar.tsx'
import Badge from '@/ui/atoms/Badge.tsx'
import { MessageSquareDot } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'
import type { ThreadDto } from '@/bindings.ts'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface ThreadListItemProps {
  thread: ThreadDto
  onSelect: (address: string) => void
}

// `latest_ms` is a wall-clock instant, not a duration. Explicit options (no seconds) rather than
// Bare `toLocaleString()`, which includes them.
const LATEST_FORMAT = new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'short' })
const formatLatest = (latestMs: bigint): string => LATEST_FORMAT.format(new Date(Number(latestMs)))

// Expects a `<ul>`/`<ol>` ancestor — ThreadList owns the list semantics, this is just the `<li>`.
const ThreadListItem = ({ thread, onSelect }: ThreadListItemProps): React.JSX.Element => {
  const handleClick = useCallback(() => {
    onSelect(thread.address)
  }, [thread.address, onSelect])

  return (
    <li>
      <button
        type="button"
        className={`relative flex w-full items-center gap-3 rounded-md border border-line px-3.5 py-2.5 text-left text-sm text-ink ${interactiveStyles.hover} ${interactiveStyles.focus}`}
        onClick={handleClick}
      >
        <Avatar />
        <span className="flex min-w-0 flex-1 flex-col items-start">
          <Text as="span">{thread.contact_name ?? thread.address}</Text>
          <Text as="span" size="xs" tone="muted">
            {formatLatest(thread.latest_ms)}
          </Text>
        </span>
        {thread.unread > 0n && <Badge count={Number(thread.unread)} icon={MessageSquareDot} position="corner" />}
      </button>
    </li>
  )
}

export default ThreadListItem
