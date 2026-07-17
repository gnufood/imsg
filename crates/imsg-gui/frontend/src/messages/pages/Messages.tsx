import type { MessageDto, ThreadDto } from '@/bindings.ts'
import MessagesTemplate from '@/messages/templates/MessagesTemplate.tsx'

interface MessagesProps {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  onResumeConversationPolling: () => void
  onResumeThreadsPolling: () => void
  onSelectThread: (address: string) => void
  selectedAddress: string | undefined
  threads: ThreadDto[] | undefined
  threadsPollFailed: boolean
}

const Messages = ({
  conversationMessages,
  conversationPollFailed,
  onResumeConversationPolling,
  onResumeThreadsPolling,
  onSelectThread,
  selectedAddress,
  threads,
  threadsPollFailed,
}: MessagesProps): React.JSX.Element => (
  <MessagesTemplate
    conversationMessages={conversationMessages}
    conversationPollFailed={conversationPollFailed}
    onResumeConversationPolling={onResumeConversationPolling}
    onResumeThreadsPolling={onResumeThreadsPolling}
    onSelectThread={onSelectThread}
    selectedAddress={selectedAddress}
    threads={threads}
    threadsPollFailed={threadsPollFailed}
  />
)

export default Messages
