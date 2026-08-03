import type { MessageDto, ThreadDto } from '@/bindings.ts'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ConversationPane from '@/messages/organisms/ConversationPane.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import ThreadListPane from '@/messages/organisms/ThreadListPane.tsx'

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

interface SplitPaneArgs {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  deleting: boolean
  leftCollapsed: boolean
  onRefreshContacts: () => void
  onRequestDelete: () => void
  onResumeConversationPolling: () => void
  onSelectThread: (address: string) => void
  onSendMessage: (text: string) => void
  onToggleLeft: () => void
  refreshingContacts: boolean
  selectedAddress: string | undefined
  selectedContactName: string | undefined
  sendError: string | undefined
  sendPending: boolean
  threads: ThreadDto[]
}

// Pulled out of `MessagesTemplate` so it stays under this repo's max-lines-per-function limit —
// Same pattern `ConversationPane`'s own `renderContent` uses.
const renderSplitPane = ({
  conversationMessages,
  conversationPollFailed,
  deleting,
  leftCollapsed,
  onRefreshContacts,
  onRequestDelete,
  onResumeConversationPolling,
  onSelectThread,
  onSendMessage,
  onToggleLeft,
  refreshingContacts,
  selectedAddress,
  selectedContactName,
  sendError,
  sendPending,
  threads,
}: SplitPaneArgs): React.JSX.Element => (
  <div className="flex h-screen w-full bg-surface text-ink">
    <ThreadListPane
      collapsed={leftCollapsed}
      onRefreshContacts={onRefreshContacts}
      onSelect={onSelectThread}
      onToggle={onToggleLeft}
      refreshingContacts={refreshingContacts}
      threads={threads}
    />
    <ConversationPane
      contactName={selectedContactName}
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

interface MessagesTemplateProps {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  deleting: boolean
  leftCollapsed: boolean
  onRefreshContacts: () => void
  onRequestDelete: () => void
  onResumeConversationPolling: () => void
  onResumeThreadsPolling: () => void
  onSelectThread: (address: string) => void
  onSendMessage: (text: string) => void
  onToggleLeft: () => void
  refreshingContacts: boolean
  selectedAddress: string | undefined
  selectedContactName: string | undefined
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
  onRefreshContacts,
  onRequestDelete,
  onResumeConversationPolling,
  onResumeThreadsPolling,
  onSelectThread,
  onSendMessage,
  onToggleLeft,
  refreshingContacts,
  selectedAddress,
  selectedContactName,
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

  return renderSplitPane({
    conversationMessages,
    conversationPollFailed,
    deleting,
    leftCollapsed,
    onRefreshContacts,
    onRequestDelete,
    onResumeConversationPolling,
    onSelectThread,
    onSendMessage,
    onToggleLeft,
    refreshingContacts,
    selectedAddress,
    selectedContactName,
    sendError,
    sendPending,
    threads,
  })
}

export default MessagesTemplate
