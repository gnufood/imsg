import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import useDaemonInstall from '@/settings/application/use-daemon-install.ts'
import useDaemonRestart from '@/settings/application/use-daemon-restart.ts'
import useDaemonServiceStatus from '@/settings/application/use-daemon-service-status.ts'
import useDaemonStop from '@/settings/application/use-daemon-stop.ts'
import useDaemonUninstall from '@/settings/application/use-daemon-uninstall.ts'

// Composes the five independent daemon-action/status hooks — each keeps its own concrete
// Pending/error state, matching this codebase's one-hook-per-capability convention (`use-send.ts`,
// `use-delete-conversation.ts`) rather than one generic parameterized hook — into
// `DaemonControlsArgs`' shape directly, so `SettingsConnected` needs one statement for all five
// Instead of five (see internal/GUI_ATOMIC_DESIGN.md, and its max-statements limit).
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
