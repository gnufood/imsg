import type UseConversationResult from '@/messages/application/use-conversation.types.ts'
import { commands } from '@/bindings.ts'
import { useEffect } from 'react'

type Messages = NonNullable<UseConversationResult['messages']>

const markUnread = (messages: Messages): Promise<unknown> => {
  const unread = messages.filter((message) => !message.read)
  return Promise.all(unread.map((message) => commands.markRead(message.handle)))
}

const useMarkRead = (messages: UseConversationResult['messages']): void => {
  useEffect(() => {
    if (messages === undefined) {
      return
    }
    void markUnread(messages)
  }, [messages])
}

export default useMarkRead
