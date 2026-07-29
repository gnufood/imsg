export default interface DaemonControlsArgs {
  installError: string | undefined
  installing: boolean
  onInstall: () => void
  onRestart: () => void
  onResumeServiceStatusPolling: () => void
  onStop: () => void
  onUninstall: () => void
  restartError: string | undefined
  restarting: boolean
  serviceStatusPollFailed: boolean
  stopError: string | undefined
  stopping: boolean
  systemInstalled: boolean | undefined
  uninstallError: string | undefined
  uninstalling: boolean
  userInstalled: boolean | undefined
}
