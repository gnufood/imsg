import type { MessageDto, ThreadDto } from '@/bindings.ts'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import ConversationView from '@/messages/organisms/ConversationView.tsx'
import EmptyState from '@/ui/molecules/EmptyState.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import MessageComposer from '@/ui/molecules/MessageComposer.tsx'
import ThreadList from '@/messages/organisms/ThreadList.tsx'

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

const paneCenter = (children: React.ReactNode): React.JSX.Element => <div className="grid h-full place-items-center p-6">{children}</div>

interface ConversationPaneArgs {
  messages: MessageDto[] | undefined
  onResumePolling: () => void
  pollFailed: boolean
  selectedAddress: string | undefined
}

const renderConversationPane = ({ messages, onResumePolling, pollFailed, selectedAddress }: ConversationPaneArgs): React.JSX.Element => {
  if (selectedAddress === undefined) {
    return paneCenter(<EmptyState message="Select a conversation." />)
  }
  if (pollFailed) {
    return paneCenter(<ErrorState message="Couldn't load this conversation." onRetry={onResumePolling} />)
  }
  if (messages === undefined) {
    return paneCenter(<LoadingState message="Loading messages…" />)
  }
  return (
    <div className="h-full w-full overflow-y-auto p-6">
      <ConversationView messages={messages} />
    </div>
  )
}

interface MessagesTemplateProps {
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

const MessagesTemplate = ({
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
}: MessagesTemplateProps): React.JSX.Element => {
  if (threadsPollFailed) {
    return screen(<ErrorState message="Couldn't reach the app backend." onRetry={onResumeThreadsPolling} />)
  }
  if (threads === undefined) {
    return screen(<LoadingState message="Loading conversations…" />)
  }

  return (
    <div className="flex h-screen w-full bg-surface text-ink">
      <div className="flex w-72 shrink-0 flex-col gap-4 overflow-y-auto border-r border-line p-4">
        <h1 className="text-sm text-muted">Conversations</h1>
        <ThreadList threads={threads} onSelect={onSelectThread} />
      </div>
      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="min-h-0 flex-1">
          {renderConversationPane({
            messages: conversationMessages,
            onResumePolling: onResumeConversationPolling,
            pollFailed: conversationPollFailed,
            selectedAddress,
          })}
        </div>
        {selectedAddress !== undefined && <MessageComposer error={sendError} onSend={onSendMessage} pending={sendPending} />}
      </div>
    </div>
  )
}

export default MessagesTemplate
