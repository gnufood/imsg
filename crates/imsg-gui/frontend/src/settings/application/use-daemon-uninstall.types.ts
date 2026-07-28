import type { UninstallResult } from '@/bindings.ts'

export default interface UseDaemonUninstallResult {
  error: string | undefined
  // What the last completed uninstall did — a removal, or a no-op because nothing was
  // Registered. `undefined` until one completes.
  outcome: UninstallResult | undefined
  uninstall: (system: boolean) => void
  uninstalling: boolean
}
