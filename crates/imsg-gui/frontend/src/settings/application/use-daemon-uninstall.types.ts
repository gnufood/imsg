import type { UninstallResult } from '@/bindings.ts'

export default interface UseDaemonUninstallResult {
  error: string | undefined
  outcome: UninstallResult | undefined
  uninstall: () => void
  uninstalling: boolean
}
