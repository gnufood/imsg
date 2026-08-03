export default interface UseDaemonStopResult {
  error: string | undefined
  stop: () => void
  stopping: boolean
}
