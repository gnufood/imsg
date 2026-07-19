export default interface UseDaemonServiceStatusResult {
  pollFailed: boolean
  resumePolling: () => void
  systemInstalled: boolean | undefined
  userInstalled: boolean | undefined
}
