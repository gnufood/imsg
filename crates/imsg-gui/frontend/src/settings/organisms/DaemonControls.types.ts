export default interface DaemonControlsArgs {
  installError: string | undefined
  installing: boolean
  onInstall: (system: boolean) => void
  onRestart: () => void
  onStop: () => void
  onUninstall: (system: boolean) => void
  restartError: string | undefined
  restarting: boolean
  stopError: string | undefined
  stopping: boolean
  uninstallError: string | undefined
  uninstalling: boolean
}
