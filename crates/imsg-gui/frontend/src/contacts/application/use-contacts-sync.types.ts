import type { SyncReportDto } from '@/bindings.ts'

export default interface UseContactsSyncResult {
  error: string | undefined
  report: SyncReportDto | undefined
  sync: () => void
  syncing: boolean
}
