import type { GateStatus } from '@/bindings.ts'

export default interface UseGateStatusResult {
  pollFailed: boolean
  proceed: () => void
  resumePolling: () => void
  status: GateStatus | undefined
}
