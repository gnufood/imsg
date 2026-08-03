import ConversationView from '@/messages/organisms/ConversationView.tsx'
import EmptyState from '@/ui/molecules/EmptyState.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import IconButton from '@/ui/atoms/IconButton.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import MessageComposer from '@/ui/molecules/MessageComposer.tsx'
import type { MessageDto } from '@/bindings.ts'
import Text from '@/ui/atoms/Text.tsx'
import { Trash2 } from 'lucide-react'

const paneCenter = (children: React.ReactNode): React.JSX.Element => <div className="grid h-full place-items-center p-6">{children}</div>

const headerLabel = (selectedAddress: string | undefined, contactName: string | undefined): string => {
  if (selectedAddress === undefined) {
    return 'Conversation'
  }
  return contactName ?? selectedAddress
}

interface ConversationContentArgs {
  messages: MessageDto[] | undefined
  onResumePolling: () => void
  pollFailed: boolean
  selectedAddress: string | undefined
}

const renderContent = ({ messages, onResumePolling, pollFailed, selectedAddress }: ConversationContentArgs): React.JSX.Element => {
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

interface ConversationPaneProps {
  contactName: string | undefined
  deleting: boolean
  messages: MessageDto[] | undefined
  onDelete: () => void
  onResumePolling: () => void
  onSendMessage: (text: string) => void
  pollFailed: boolean
  selectedAddress: string | undefined
  sendError: string | undefined
  sendPending: boolean
}

const ConversationPane = ({
  contactName,
  deleting,
  messages,
  onDelete,
  onResumePolling,
  onSendMessage,
  pollFailed,
  selectedAddress,
  sendError,
  sendPending,
}: ConversationPaneProps): React.JSX.Element => (
  <div className="flex flex-1 flex-col overflow-hidden">
    <div className="flex items-center justify-between border-b border-line p-4">
      <Text as="span" tone="muted">
        {headerLabel(selectedAddress, contactName)}
      </Text>
      {selectedAddress !== undefined && <IconButton disabled={deleting} icon={Trash2} label="Delete conversation" onClick={onDelete} />}
    </div>
    <div className="min-h-0 flex-1">{renderContent({ messages, onResumePolling, pollFailed, selectedAddress })}</div>
    {selectedAddress !== undefined && <MessageComposer error={sendError} onSend={onSendMessage} pending={sendPending} />}
  </div>
)

export default ConversationPane
