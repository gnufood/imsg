import { useCallback, useState } from 'react'
import Messages from '@/messages/pages/Messages.tsx'
import useConversation from '@/messages/application/use-conversation.ts'
import useMarkRead from '@/messages/application/use-mark-read.ts'
import useSend from '@/messages/application/use-send.ts'
import useThreads from '@/messages/application/use-threads.ts'

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useThreads`/`useConversation`/`useSend`/`useMarkRead`'s real backend calls and `Messages`'s
// Presentational page. Owns `selectedAddress` itself — it's the one piece of state shared across
// The hooks (which thread's messages to poll/send/mark-read into), not local to any of them.
const MessagesConnected = (): React.JSX.Element => {
  const { pollFailed: threadsPollFailed, resumePolling: resumeThreadsPolling, threads } = useThreads()
  const [selectedAddress, setSelectedAddress] = useState<string | undefined>()
  const { messages, pollFailed: conversationPollFailed, resumePolling: resumeConversationPolling } = useConversation(selectedAddress)
  const { error: sendError, send, sending: sendPending } = useSend(selectedAddress)
  useMarkRead(messages)

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
