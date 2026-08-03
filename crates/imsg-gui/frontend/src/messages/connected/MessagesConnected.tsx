import { useCallback, useState } from 'react'
import Messages from '@/messages/pages/Messages.tsx'
import useContactsSync from '@/contacts/application/use-contacts-sync.ts'
import useConversation from '@/messages/application/use-conversation.ts'
import useDeleteConversation from '@/messages/application/use-delete-conversation.ts'
import useMarkRead from '@/messages/application/use-mark-read.ts'
import useSend from '@/messages/application/use-send.ts'
import useThreads from '@/messages/application/use-threads.ts'

const MessagesConnected = (): React.JSX.Element => {
  const { pollFailed: threadsPollFailed, resumePolling: resumeThreadsPolling, threads } = useThreads()
  const [selectedAddress, setSelectedAddress] = useState<string | undefined>()
  const { messages, pollFailed: conversationPollFailed, resumePolling: resumeConversationPolling } = useConversation(selectedAddress)
  const { error: sendError, send, sending: sendPending } = useSend(selectedAddress)
  const { sync: refreshContacts, syncing: refreshingContacts } = useContactsSync()
  useMarkRead(messages)

  const handleDeleted = useCallback(() => {
    setSelectedAddress(undefined)
  }, [])

  const {
    cancelDelete,
    confirmDelete,
    confirmOpen: deleteConfirmOpen,
    deleting,
    error: deleteError,
    requestDelete,
  } = useDeleteConversation(selectedAddress, messages, handleDeleted)

  const handleSelectThread = useCallback((address: string) => {
    setSelectedAddress(address)
  }, [])

  return (
    <Messages
      conversationMessages={messages}
      conversationPollFailed={conversationPollFailed}
      deleteConfirmOpen={deleteConfirmOpen}
      deleteError={deleteError}
      deleting={deleting}
      onCancelDelete={cancelDelete}
      onConfirmDelete={confirmDelete}
      onRefreshContacts={refreshContacts}
      onRequestDelete={requestDelete}
      onResumeConversationPolling={resumeConversationPolling}
      onResumeThreadsPolling={resumeThreadsPolling}
      onSelectThread={handleSelectThread}
      onSendMessage={send}
      refreshingContacts={refreshingContacts}
      selectedAddress={selectedAddress}
      sendError={sendError}
      sendPending={sendPending}
      threads={threads}
      threadsPollFailed={threadsPollFailed}
    />
  )
}

export default MessagesConnected
