import type { SyncReportDto } from '@/bindings.ts'

export default interface UseContactsSyncResult {
  error: string | undefined
  // What the last completed sync did — an untouched cache, or a refresh and what it stored.
  // `undefined` until one completes.
  report: SyncReportDto | undefined
  sync: () => void
  syncing: boolean
}
