import { useCallback, useState } from 'react'
import Messages from '@/messages/pages/Messages.tsx'
import useConversation from '@/messages/application/use-conversation.ts'
import useSend from '@/messages/application/use-send.ts'
import useThreads from '@/messages/application/use-threads.ts'

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useThreads`/`useConversation`/`useSend`'s real backend calls and `Messages`'s presentational
// Page. Owns `selectedAddress` itself — it's the one piece of state shared across the hooks
// (which thread's messages to poll/send into), not local to any of them.
const MessagesConnected = (): React.JSX.Element => {
  const { pollFailed: threadsPollFailed, resumePolling: resumeThreadsPolling, threads } = useThreads()
  const [selectedAddress, setSelectedAddress] = useState<string | undefined>()
  const { messages, pollFailed: conversationPollFailed, resumePolling: resumeConversationPolling } = useConversation(selectedAddress)
  const { error: sendError, send, sending: sendPending } = useSend(selectedAddress)

  const handleSelectThread = useCallback((address: string) => {
    setSelectedAddress(address)
  }, [])

  return (
    <Messages
      conversationMessages={messages}
      conversationPollFailed={conversationPollFailed}
      onResumeConversationPolling={resumeConversationPolling}
      onResumeThreadsPolling={resumeThreadsPolling}
      onSelectThread={handleSelectThread}
      onSendMessage={send}
      selectedAddress={selectedAddress}
      sendError={sendError}
      sendPending={sendPending}
      threads={threads}
      threadsPollFailed={threadsPollFailed}
    />
  )
}

export default MessagesConnected
