import type { MessageDto } from '@/bindings.ts'

export default interface UseConversationResult {
  messages: MessageDto[] | undefined
  pollFailed: boolean
  resumePolling: () => void
}
