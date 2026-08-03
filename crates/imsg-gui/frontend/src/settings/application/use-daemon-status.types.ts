import type { SessionState } from '@/bindings.ts'

export default interface UseDaemonStatusResult {
  pollFailed: boolean
  resumePolling: () => void
  status: SessionState | null | undefined
}
