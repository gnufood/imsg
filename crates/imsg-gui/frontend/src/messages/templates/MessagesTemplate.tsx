import type { MessageDto, ThreadDto } from '@/bindings.ts'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ConversationPane from '@/messages/organisms/ConversationPane.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import ThreadListPane from '@/messages/organisms/ThreadListPane.tsx'

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

interface MessagesTemplateProps {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  deleting: boolean
  leftCollapsed: boolean
  onRequestDelete: () => void
  onResumeConversationPolling: () => void
  onResumeThreadsPolling: () => void
  onSelectThread: (address: string) => void
  onSendMessage: (text: string) => void
  onToggleLeft: () => void
  selectedAddress: string | undefined
  sendError: string | undefined
  sendPending: boolean
  threads: ThreadDto[] | undefined
  threadsPollFailed: boolean
}

const MessagesTemplate = ({
  conversationMessages,
  conversationPollFailed,
  deleting,
  leftCollapsed,
  onRequestDelete,
  onResumeConversationPolling,
  onResumeThreadsPolling,
  onSelectThread,
  onSendMessage,
  onToggleLeft,
  selectedAddress,
  sendError,
  sendPending,
  threads,
  threadsPollFailed,
}: MessagesTemplateProps): React.JSX.Element => {
  if (threadsPollFailed) {
    return screen(<ErrorState message="Couldn't reach the app backend." onRetry={onResumeThreadsPolling} />)
  }
  if (threads === undefined) {
    return screen(<LoadingState message="Loading conversations…" />)
  }

  return (
    <div className="flex h-screen w-full bg-surface text-ink">
      <ThreadListPane collapsed={leftCollapsed} onSelect={onSelectThread} onToggle={onToggleLeft} threads={threads} />
      <ConversationPane
        deleting={deleting}
        messages={conversationMessages}
        onDelete={onRequestDelete}
        onResumePolling={onResumeConversationPolling}
        onSendMessage={onSendMessage}
        pollFailed={conversationPollFailed}
        selectedAddress={selectedAddress}
        sendError={sendError}
        sendPending={sendPending}
      />
    </div>
  )
}

export default MessagesTemplate
