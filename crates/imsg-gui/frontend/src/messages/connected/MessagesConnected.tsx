import { useCallback, useState } from 'react'
import Messages from '@/messages/pages/Messages.tsx'
import useConversation from '@/messages/application/use-conversation.ts'
import useThreads from '@/messages/application/use-threads.ts'

// Production IPC-connected wrapper (see internal/GUI_ATOMIC_DESIGN.md) — the seam between
// `useThreads`/`useConversation`'s real backend polling and `Messages`'s presentational page.
// Owns `selectedAddress` itself — it's the one piece of state shared between the two hooks
// (which thread's messages `useConversation` should be polling), not local to either.
const MessagesConnected = (): React.JSX.Element => {
  const { pollFailed: threadsPollFailed, resumePolling: resumeThreadsPolling, threads } = useThreads()
  const [selectedAddress, setSelectedAddress] = useState<string | undefined>()
  const { messages, pollFailed: conversationPollFailed, resumePolling: resumeConversationPolling } = useConversation(selectedAddress)

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
      selectedAddress={selectedAddress}
      threads={threads}
      threadsPollFailed={threadsPollFailed}
    />
  )
}

export default MessagesConnected
