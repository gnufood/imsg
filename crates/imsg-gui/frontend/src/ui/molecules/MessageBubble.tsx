import type { MessageDto } from '@/bindings.ts'
import Text from '@/ui/atoms/Text.tsx'

interface MessageBubbleProps {
  message: MessageDto
}

// Explicit options (no seconds) rather than bare `toLocaleString()`, which includes them.
const TIMESTAMP_FORMAT = new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'short' })
const formatTimestamp = (timestampMs: bigint): string => TIMESTAMP_FORMAT.format(new Date(Number(timestampMs)))

// Mirrors `store::OutgoingStatus`'s variants (see bindings.ts) — `None` (received messages)
// Never reaches this map, guarded by the `outgoing_status !== null` check below.
const OUTGOING_LABEL: Record<NonNullable<MessageDto['outgoing_status']>, string> = {
  FailedPermanent: 'Failed to send',
  FailedRetryable: 'Failed, retrying…',
  Queued: 'Queued',
  Sending: 'Sending…',
  SentConfirmed: 'Delivered',
  SentUnconfirmed: 'Sent',
  Unknown: 'Delivery unknown',
}

// Expects a `<ul>`/`<ol>` ancestor — ConversationView owns the list semantics, this is just
// The `<li>`.
const MessageBubble = ({ message }: MessageBubbleProps): React.JSX.Element => {
  if (message.direction === 'Sent') {
    return (
      <li className="flex flex-col items-end">
        <div className="max-w-[75%] rounded-lg bg-ink px-3 py-2">
          <Text tone="surface">{message.text}</Text>
        </div>
        <Text size="xs" tone="muted">
          {formatTimestamp(message.timestamp_ms)}
          {message.outgoing_status !== null && ` · ${OUTGOING_LABEL[message.outgoing_status]}`}
        </Text>
      </li>
    )
  }

  return (
    <li className="flex flex-col items-start">
      <div className="max-w-[75%] rounded-lg border border-line px-3 py-2">
        <Text tone="ink">{message.text}</Text>
      </div>
      <Text size="xs" tone="muted">
        {formatTimestamp(message.timestamp_ms)}
      </Text>
    </li>
  )
}

export default MessageBubble
