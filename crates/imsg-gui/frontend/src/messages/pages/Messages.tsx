import type { MessageDto, ThreadDto } from '@/bindings.ts'
import MessagesTemplate from '@/messages/templates/MessagesTemplate.tsx'

interface MessagesProps {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  onResumeConversationPolling: () => void
  onResumeThreadsPolling: () => void
  onSelectThread: (address: string) => void
  onSendMessage: (text: string) => void
  selectedAddress: string | undefined
  sendError: string | undefined
  sendPending: boolean
  threads: ThreadDto[] | undefined
  threadsPollFailed: boolean
}

const Messages = ({
  conversationMessages,
  conversationPollFailed,
  onResumeConversationPolling,
  onResumeThreadsPolling,
  onSelectThread,
  onSendMessage,
  selectedAddress,
  sendError,
  sendPending,
  threads,
  threadsPollFailed,
}: MessagesProps): React.JSX.Element => (
  <MessagesTemplate
    conversationMessages={conversationMessages}
    conversationPollFailed={conversationPollFailed}
    onResumeConversationPolling={onResumeConversationPolling}
    onResumeThreadsPolling={onResumeThreadsPolling}
    onSelectThread={onSelectThread}
    onSendMessage={onSendMessage}
    selectedAddress={selectedAddress}
    sendError={sendError}
    sendPending={sendPending}
    threads={threads}
    threadsPollFailed={threadsPollFailed}
  />
)

export default Messages
