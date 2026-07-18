import type { MessageDto, ThreadDto } from '@/bindings.ts'
import { useCallback, useState } from 'react'
import { AnimatePresence } from 'motion/react'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import MessagesTemplate from '@/messages/templates/MessagesTemplate.tsx'

interface DeleteDialogArgs {
  deleteError: string | undefined
  deleting: boolean
  onCancelDelete: () => void
  onConfirmDelete: () => void
  selectedAddress: string | undefined
}

const renderDeleteDialog = ({ deleteError, deleting, onCancelDelete, onConfirmDelete, selectedAddress }: DeleteDialogArgs): React.JSX.Element => (
  <ConfirmDialog
    confirmLabel="Delete"
    error={deleteError}
    message={`Delete all messages with ${selectedAddress ?? 'this contact'}? This can't be undone.`}
    onCancel={onCancelDelete}
    onConfirm={onConfirmDelete}
    pending={deleting}
    title="Delete conversation?"
  />
)

interface PaneCollapseState {
  leftCollapsed: boolean
  toggleLeft: () => void
}

// Pulled out of the component so `Messages` itself stays under this repo's max-lines-per-function
// Limit — still page-owned, presentation-only state, just packaged as a local hook.
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
  onRequestDelete: () => void
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

// Pane collapse/expand is presentation-only, per-session UI state — never persisted, never
// Shared with the polling hooks — so it lives here rather than in `MessagesConnected`. The
// Delete-confirm dialog's open/pending/error state, by contrast, comes from `useDeleteConversation`
// (an IPC-backed hook), so only its rendering — not its state — lives here.
const Messages = ({
  conversationMessages,
  conversationPollFailed,
  deleteConfirmOpen,
  deleteError,
  deleting,
  onCancelDelete,
  onConfirmDelete,
  onRequestDelete,
  onResumeConversationPolling,
  onResumeThreadsPolling,
  onSelectThread,
  onSendMessage,
  selectedAddress,
  sendError,
  sendPending,
  threads,
  threadsPollFailed,
}: MessagesProps): React.JSX.Element => {
  const { leftCollapsed, toggleLeft } = usePaneCollapse()

  return (
    <>
      <MessagesTemplate
        conversationMessages={conversationMessages}
        conversationPollFailed={conversationPollFailed}
        deleting={deleting}
        leftCollapsed={leftCollapsed}
        onRequestDelete={onRequestDelete}
        onResumeConversationPolling={onResumeConversationPolling}
        onResumeThreadsPolling={onResumeThreadsPolling}
        onSelectThread={onSelectThread}
        onSendMessage={onSendMessage}
        onToggleLeft={toggleLeft}
        selectedAddress={selectedAddress}
        sendError={sendError}
        sendPending={sendPending}
        threads={threads}
        threadsPollFailed={threadsPollFailed}
      />
      <AnimatePresence>
        {deleteConfirmOpen && renderDeleteDialog({ deleteError, deleting, onCancelDelete, onConfirmDelete, selectedAddress })}
      </AnimatePresence>
    </>
  )
}

export default Messages
