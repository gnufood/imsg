import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import useDaemonInstall from '@/settings/application/use-daemon-install.ts'
import useDaemonRestart from '@/settings/application/use-daemon-restart.ts'
import useDaemonStop from '@/settings/application/use-daemon-stop.ts'
import useDaemonUninstall from '@/settings/application/use-daemon-uninstall.ts'

// Composes the four independent daemon-action hooks — each keeps its own concrete pending/error
// State, matching this codebase's one-hook-per-capability convention (`use-send.ts`,
// `use-delete-conversation.ts`) rather than one generic parameterized hook — into
// `DaemonControlsArgs`' shape directly, so `SettingsConnected` needs one statement for all four
// Instead of four (see internal/GUI_ATOMIC_DESIGN.md, and its max-statements limit).
const useDaemonActions = (addr: string | undefined, onChanged: () => void): DaemonControlsArgs => {
  const { error: stopError, stop, stopping } = useDaemonStop(addr, onChanged)
  const { error: restartError, restart, restarting } = useDaemonRestart(addr, onChanged)
  const { error: installError, install, installing } = useDaemonInstall(addr, onChanged)
  const { error: uninstallError, uninstall, uninstalling } = useDaemonUninstall(onChanged)

  return {
    installError,
    installing,
    onInstall: install,
    onRestart: restart,
    onStop: stop,
    onUninstall: uninstall,
    restartError,
    restarting,
    stopError,
    stopping,
    uninstallError,
    uninstalling,
  }
}

export default useDaemonActions
