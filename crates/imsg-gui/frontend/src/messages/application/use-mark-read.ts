import type UseConversationResult from '@/messages/application/use-conversation.types.ts'
import { commands } from '@/bindings.ts'
import { useEffect } from 'react'

type Messages = NonNullable<UseConversationResult['messages']>

// A failed `markRead` here is silently retried on the next poll tick (any message still
// `read: false` gets re-submitted) rather than surfaced — there's no UI for it to report into,
// And an unread badge staying stale for one extra 5s cycle isn't worth inventing one for.
const markUnread = (messages: Messages): Promise<unknown> => {
  const unread = messages.filter((message) => !message.read)
  return Promise.all(unread.map((message) => commands.markRead(message.handle)))
}

// Application boundary for the messages feature slice (see internal/GUI_ATOMIC_DESIGN.md) — the
// Only file here (alongside use-threads.ts/use-conversation.ts/use-send.ts) allowed to import
// Bindings.ts. No return value: this is a pure side effect, not state any component renders.
const useMarkRead = (messages: UseConversationResult['messages']): void => {
  useEffect(() => {
    if (messages === undefined) {
      return
    }
    void markUnread(messages)
  }, [messages])
}

export default useMarkRead
