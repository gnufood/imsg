export default interface UseDaemonInstallResult {
  error: string | undefined
  install: (system: boolean) => void
  installing: boolean
}
