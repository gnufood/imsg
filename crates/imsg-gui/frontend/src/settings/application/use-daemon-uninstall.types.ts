export default interface UseDaemonUninstallResult {
  error: string | undefined
  uninstall: (system: boolean) => void
  uninstalling: boolean
}
