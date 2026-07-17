import type { ThreadDto } from '@/bindings.ts'

export default interface UseThreadsResult {
  pollFailed: boolean
  resumePolling: () => void
  threads: ThreadDto[] | undefined
}
