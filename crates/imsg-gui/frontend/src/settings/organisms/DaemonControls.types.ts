export default interface DaemonControlsArgs {
  installError: string | undefined
  installing: boolean
  onInstall: (system: boolean) => void
  onRestart: () => void
  onResumeServiceStatusPolling: () => void
  onStop: () => void
  onUninstall: (system: boolean) => void
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
