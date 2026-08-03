import type { MessageDto, ThreadDto } from '@/bindings.ts'
import { useCallback, useMemo, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import MessagesTemplate from '@/messages/templates/MessagesTemplate.tsx'

interface DeleteDialogArgs {
  deleteError: string | undefined
  deleting: boolean
  onCancelDelete: () => void
  onConfirmDelete: () => void
  selectedContactName: string | undefined
}

const renderDeleteDialog = ({ deleteError, deleting, onCancelDelete, onConfirmDelete, selectedContactName }: DeleteDialogArgs): React.JSX.Element => (
  <ConfirmDialog
    confirmLabel="Delete"
    error={deleteError}
    message={`Delete all messages with ${selectedContactName ?? 'this contact'}? This can't be undone.`}
    onCancel={onCancelDelete}
    onConfirm={onConfirmDelete}
    pending={deleting}
    title="Delete conversation?"
  />
)

interface MessagesTemplateArgs {
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

const renderMessagesTemplate = ({
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
}: MessagesTemplateArgs): React.JSX.Element => (
  <MessagesTemplate
    conversationMessages={conversationMessages}
    conversationPollFailed={conversationPollFailed}
    deleting={deleting}
    leftCollapsed={leftCollapsed}
    onRefreshContacts={onRefreshContacts}
    onRequestDelete={onRequestDelete}
    onResumeConversationPolling={onResumeConversationPolling}
    onResumeThreadsPolling={onResumeThreadsPolling}
    onSelectThread={onSelectThread}
    onSendMessage={onSendMessage}
    onToggleLeft={onToggleLeft}
    refreshingContacts={refreshingContacts}
    selectedAddress={selectedAddress}
    selectedContactName={selectedContactName}
    sendError={sendError}
    sendPending={sendPending}
    threads={threads}
    threadsPollFailed={threadsPollFailed}
  />
)

interface PaneCollapseState {
  leftCollapsed: boolean
  toggleLeft: () => void
}

const usePaneCollapse = (): PaneCollapseState => {
  const [leftCollapsed, setLeftCollapsed] = useState(false)

  const toggleLeft = useCallback(() => {
    setLeftCollapsed((collapsed) => !collapsed)
  }, [])

  return { leftCollapsed, toggleLeft }
}

interface MessagesProps {
  conversationMessages: MessageDto[] | undefined
  conversationPollFailed: boolean
  deleteConfirmOpen: boolean
  deleteError: string | undefined
  deleting: boolean
  onCancelDelete: () => void
  onConfirmDelete: () => void
  onRefreshContacts: () => void
  onRequestDelete: () => void
  onResumeConversationPolling: () => void
  onResumeThreadsPolling: () => void
  onSelectThread: (address: string) => void
  onSendMessage: (text: string) => void
  refreshingContacts: boolean
  selectedAddress: string | undefined
  sendError: string | undefined
  sendPending: boolean
  threads: ThreadDto[] | undefined
  threadsPollFailed: boolean
}

const useSelectedContactName = (threads: ThreadDto[] | undefined, selectedAddress: string | undefined): string | undefined =>
  useMemo(() => threads?.find((thread) => thread.address === selectedAddress)?.contact_name ?? undefined, [threads, selectedAddress])

interface MessagesPageArgs extends MessagesProps {
  leftCollapsed: boolean
  selectedContactName: string | undefined
  toggleLeft: () => void
}

const renderMessagesPage = ({
  deleteConfirmOpen,
  deleteError,
  deleting,
  leftCollapsed,
  onCancelDelete,
  onConfirmDelete,
  selectedContactName,
  toggleLeft,
  ...templateArgs
}: MessagesPageArgs): React.JSX.Element => (
  <>
    {renderMessagesTemplate({ ...templateArgs, deleting, leftCollapsed, onToggleLeft: toggleLeft, selectedContactName })}
    <AnimatePresence>
      {deleteConfirmOpen && renderDeleteDialog({ deleteError, deleting, onCancelDelete, onConfirmDelete, selectedContactName })}
    </AnimatePresence>
  </>
)

const Messages = (props: MessagesProps): React.JSX.Element => {
  const { leftCollapsed, toggleLeft } = usePaneCollapse()
  const selectedContactName = useSelectedContactName(props.threads, props.selectedAddress)
  return renderMessagesPage({ ...props, leftCollapsed, selectedContactName, toggleLeft })
}

export default Messages
