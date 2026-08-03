export default interface UseDaemonRestartResult {
  error: string | undefined
  restart: () => void
  restarting: boolean
}
