export default interface UseDaemonInstallResult {
  error: string | undefined
  install: () => void
  installing: boolean
}
