import EmptyState from '@/ui/molecules/EmptyState.tsx'
import MessageBubble from '@/ui/molecules/MessageBubble.tsx'
import type { MessageDto } from '@/bindings.ts'

interface ConversationViewProps {
  messages: MessageDto[]
}

const ConversationView = ({ messages }: ConversationViewProps): React.JSX.Element => {
  if (messages.length === 0) {
    return <EmptyState message="No messages yet." />
  }

  return (
    <ul className="flex w-full flex-col gap-3">
      {messages.map((message) => (
        <MessageBubble key={message.handle} message={message} />
      ))}
    </ul>
  )
}

export default ConversationView
