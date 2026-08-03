export default interface UseContactsSyncResult {
  error: string | undefined
  sync: () => void
  synced: number | undefined
  syncing: boolean
}
