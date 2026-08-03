import type { SessionState } from '@/bindings.ts'

export default interface StatusPanelArgs {
  address: string | undefined
  configFailed: boolean
  onResumeStatusPolling: () => void
  onRetryConfig: () => void
  status: SessionState | null | undefined
  statusPollFailed: boolean
}
