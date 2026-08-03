import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import useDaemonInstall from '@/settings/application/use-daemon-install.ts'
import useDaemonRestart from '@/settings/application/use-daemon-restart.ts'
import useDaemonServiceStatus from '@/settings/application/use-daemon-service-status.ts'
import useDaemonStop from '@/settings/application/use-daemon-stop.ts'
import useDaemonUninstall from '@/settings/application/use-daemon-uninstall.ts'

const useDaemonActions = (addr: string | undefined, onChanged: () => void): DaemonControlsArgs => {
  const { error: stopError, stop, stopping } = useDaemonStop(addr, onChanged)
  const { error: restartError, restart, restarting } = useDaemonRestart(addr, onChanged)
  const { error: installError, install, installing } = useDaemonInstall(addr, onChanged)
  const { error: uninstallError, uninstall, uninstalling } = useDaemonUninstall(onChanged)
  const { pollFailed: serviceStatusPollFailed, resumePolling, systemInstalled, userInstalled } = useDaemonServiceStatus()

  return {
    installError,
    installing,
    onInstall: install,
    onRestart: restart,
    onResumeServiceStatusPolling: resumePolling,
    onStop: stop,
    onUninstall: uninstall,
    restartError,
    restarting,
    serviceStatusPollFailed,
    stopError,
    stopping,
    systemInstalled,
    uninstallError,
    uninstalling,
    userInstalled,
  }
}

export default useDaemonActions
