import type { Meta, StoryObj } from '@storybook/react-vite'
import { useCallback, useState } from 'react'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import Contacts from '@/contacts/pages/Contacts.tsx'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import Messages from '@/messages/pages/Messages.tsx'
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

const contactsSlot = (
  <Contacts
    contacts={[
      { display_name: 'Jane Doe', uid: 'U1' },
      { display_name: null, uid: 'U2' },
    ]}
    detailContact={{ display_name: 'Jane Doe', phones: ['+15550001'], uid: 'U1' }}
    detailEmptyMessage="Select a contact from the list."
    detailFailed={false}
    detailLoading={false}
    hasNextPage={false}
    hasPrevPage={false}
    listFailed={false}
    onClearSearch={fn()}
    onNextPage={fn()}
    onPrevPage={fn()}
    onRefresh={fn()}
    onRetryDetail={fn()}
    onRetryList={fn()}
    onSearch={fn()}
    onSelectContact={fn()}
    refreshing={false}
    searchActive={false}
    searchError={undefined}
    searching={false}
    syncError={undefined}
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

interface DaemonSimState {
  installing: boolean
  systemInstalled: boolean
  uninstalling: boolean
  userInstalled: boolean
}

const DAEMON_SIM_INITIAL: DaemonSimState = { installing: false, systemInstalled: false, uninstalling: false, userInstalled: false }

// Simulates the real round trip `use-daemon-actions.ts` drives (brief pending state, then the
// Installed flag flips) — this is the one story where `AppTemplate` renders the real `Settings`
// Page, so Install/Uninstall need to actually do something instead of just logging to the
// Actions panel. `DaemonControls` itself already owns the uninstall confirm-dialog step; this
// Only has to react once that's resolved into a real `onUninstall` call.
const SIMULATED_DAEMON_DELAY_MS = 400

const useSimulatedDaemonControls = (): DaemonControlsArgs => {
  const [state, setState] = useState(DAEMON_SIM_INITIAL)

  const onInstall = useCallback((system: boolean) => {
    setState((current) => ({ ...current, installing: true }))
    setTimeout(() => {
      setState((current) => {
        if (system) {
          return { ...current, installing: false, systemInstalled: true }
        }
        return { ...current, installing: false, userInstalled: true }
      })
    }, SIMULATED_DAEMON_DELAY_MS)
  }, [])

  const onUninstall = useCallback((system: boolean) => {
    setState((current) => ({ ...current, uninstalling: true }))
    setTimeout(() => {
      setState((current) => {
        if (system) {
          return { ...current, systemInstalled: false, uninstalling: false }
        }
        return { ...current, uninstalling: false, userInstalled: false }
      })
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
    systemInstalled: state.systemInstalled,
    uninstallError: undefined,
    uninstalling: state.uninstalling,
    userInstalled: state.userInstalled,
  }
}

const InteractiveSettingsSlot = (): React.JSX.Element => {
  const daemonControls = useSimulatedDaemonControls()
  return <Settings appearance={appearanceArgs} channelOverrides={channelOverridesArgs} daemonControls={daemonControls} statusPanel={statusPanelArgs} />
}

const settingsSlot = <InteractiveSettingsSlot />

const meta = {
  args: {
    contactsSlot,
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
      contactsSlot={args.contactsSlot}
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
