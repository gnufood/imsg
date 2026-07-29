import type { Meta, StoryObj } from '@storybook/react-vite'
import { useCallback, useState } from 'react'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import Messages from '@/messages/pages/Messages.tsx'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import Settings from '@/settings/pages/Settings.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'

// Real pages, not placeholders — this is the one story showing the whole app (nav included)
// The way it actually looks past the gate, since `App.tsx` itself can't be storied (it renders
// Real-IPC `MessagesConnected`/`SettingsConnected`, same reason `GateTemplate` exists).
const messagesSlot = (
  <Messages
    conversationMessages={[
      {
        address: '00:11:22:33:44:55',
        direction: 'Received',
        folder: 'telecom/msg/inbox',
        handle: '1001',
        outgoing_status: null,
        read: true,
        text: 'Hey, are we still on for tonight?',
        timestamp_ms: 1_752_700_800_000n,
      },
      {
        address: '00:11:22:33:44:55',
        direction: 'Sent',
        folder: 'telecom/msg/sent',
        handle: '1002',
        outgoing_status: 'SentConfirmed',
        read: true,
        text: 'Yep, see you at 7.',
        timestamp_ms: 1_752_700_860_000n,
      },
    ]}
    conversationPollFailed={false}
    deleteConfirmOpen={false}
    deleteError={undefined}
    deleting={false}
    onCancelDelete={fn()}
    onConfirmDelete={fn()}
    onRefreshContacts={fn()}
    onRequestDelete={fn()}
    onResumeConversationPolling={fn()}
    onResumeThreadsPolling={fn()}
    onSelectThread={fn()}
    onSendMessage={fn()}
    refreshingContacts={false}
    selectedAddress="00:11:22:33:44:55"
    sendError={undefined}
    sendPending={false}
    threads={[
      { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
      { address: 'AA:BB:CC:DD:EE:FF', contact_name: 'Jane Doe', latest_ms: 1_752_614_400_000n, latest_outgoing_status: 'SentConfirmed', total: 4n, unread: 0n },
    ]}
    threadsPollFailed={false}
  />
)

const appearanceArgs = {
  onPreferenceChange: fn(),
  preference: 'system' as const,
}

const channelOverridesArgs = {
  detectError: undefined,
  detecting: false,
  mapChannel: 8,
  mapDraft: '8',
  onCancel: fn(),
  onDetect: fn(),
  onMapDraftChange: fn(),
  onPbapDraftChange: fn(),
  onSave: fn(),
  pbapChannel: 12,
  pbapDraft: '12',
  saveError: undefined,
  saving: false,
}

const statusPanelArgs = {
  address: '00:11:22:33:44:55',
  configFailed: false,
  onResumeStatusPolling: fn(),
  onRetryConfig: fn(),
  status: 'active' as const,
  statusPollFailed: false,
}

interface SecuritySimState {
  committedLevel: SecurityLevelArgs['draft']
  draft: SecurityLevelArgs['draft']
  saving: boolean
}

const SECURITY_SIM_INITIAL: SecuritySimState = { committedLevel: 'Medium', draft: 'Medium', saving: false }
const SECURITY_SIM_DELAY_MS = 400

// Same reasoning as `useSimulatedDaemonControls` below — `SecurityLevelForm` is fully prop-driven
// (no internal state of its own), so static `fn()` args would leave its segments looking dead in
// This one story where the real `Settings` page is rendered instead of fixture args elsewhere.
const useSimulatedSecurityLevel = (): SecurityLevelArgs => {
  const [state, setState] = useState(SECURITY_SIM_INITIAL)

  const onDraftChange = useCallback((draft: SecurityLevelArgs['draft']) => {
    setState((current) => ({ ...current, draft }))
  }, [])

  const onCancel = useCallback(() => {
    setState((current) => ({ ...current, draft: current.committedLevel }))
  }, [])

  const onSave = useCallback(() => {
    setState((current) => ({ ...current, saving: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, committedLevel: current.draft, saving: false }))
    }, SECURITY_SIM_DELAY_MS)
  }, [])

  return { committedLevel: state.committedLevel, draft: state.draft, onCancel, onDraftChange, onSave, saveError: undefined, saving: state.saving }
}

interface DaemonSimState {
  installing: boolean
  uninstalling: boolean
  userInstalled: boolean
}

const DAEMON_SIM_INITIAL: DaemonSimState = { installing: false, uninstalling: false, userInstalled: false }

// Simulates the real round trip `use-daemon-actions.ts` drives (brief pending state, then the
// Installed flag flips) — this is the one story where `AppTemplate` renders the real `Settings`
// Page, so Install/Uninstall need to actually do something instead of just logging to the
// Actions panel. `DaemonControls` itself already owns the uninstall confirm-dialog step; this
// Only has to react once that's resolved into a real `onUninstall` call.
const SIMULATED_DAEMON_DELAY_MS = 400

const useSimulatedDaemonControls = (): DaemonControlsArgs => {
  const [state, setState] = useState(DAEMON_SIM_INITIAL)

  const onInstall = useCallback(() => {
    setState((current) => ({ ...current, installing: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, installing: false, userInstalled: true }))
    }, SIMULATED_DAEMON_DELAY_MS)
  }, [])

  const onUninstall = useCallback(() => {
    setState((current) => ({ ...current, uninstalling: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, uninstalling: false, userInstalled: false }))
    }, SIMULATED_DAEMON_DELAY_MS)
  }, [])

  return {
    installError: undefined,
    installing: state.installing,
    onInstall,
    onRestart: fn(),
    onResumeServiceStatusPolling: fn(),
    onStop: fn(),
    onUninstall,
    restartError: undefined,
    restarting: false,
    serviceStatusPollFailed: false,
    stopError: undefined,
    stopping: false,
    // Status-only in `DaemonControls`; the simulated round trip below never flips it.
    systemInstalled: false,
    uninstallError: undefined,
    uninstalling: state.uninstalling,
    userInstalled: state.userInstalled,
  }
}

const InteractiveSettingsSlot = (): React.JSX.Element => {
  const daemonControls = useSimulatedDaemonControls()
  const securityLevel = useSimulatedSecurityLevel()
  return (
    <Settings
      appearance={appearanceArgs}
      channelOverrides={channelOverridesArgs}
      daemonControls={daemonControls}
      securityLevel={securityLevel}
      statusPanel={statusPanelArgs}
    />
  )
}

const settingsSlot = <InteractiveSettingsSlot />

const meta = {
  args: {
    messagesSlot,
    onSelectScreen: fn(),
    screen: 'messages',
    settingsSlot,
  },
  component: AppTemplate,
  parameters: { layout: 'fullscreen' },
  title: 'templates/AppTemplate',
} satisfies Meta<typeof AppTemplate>

export default meta
type Story = StoryObj<typeof meta>

type AppTemplateProps = React.ComponentProps<typeof AppTemplate>

// `onSelectScreen: fn()` alone only lets Storybook's Actions panel log the click — it never
// Feeds back into `screen`, so the nav would look dead. `useArgs` closes that loop: clicking
// Either nav icon updates the story's own `screen` arg, so both directions actually switch.
const InteractiveAppTemplate = (): React.JSX.Element => {
  const [args, updateArgs] = useArgs<AppTemplateProps>()

  const handleSelectScreen: AppTemplateProps['onSelectScreen'] = useCallback(
    (screen) => {
      args.onSelectScreen(screen)
      updateArgs({ screen })
    },
    [args, updateArgs],
  )

  return (
    <AppTemplate
      messagesSlot={args.messagesSlot}
      onSelectScreen={handleSelectScreen}
      screen={args.screen}
      settingsSlot={args.settingsSlot}
    />
  )
}

export const Interactive: Story = {
  render: InteractiveAppTemplate,
}
