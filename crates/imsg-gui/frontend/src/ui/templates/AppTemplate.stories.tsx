import type { Meta, StoryObj } from '@storybook/react-vite'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import Messages from '@/messages/pages/Messages.tsx'
import Settings from '@/settings/pages/Settings.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'
import { useCallback } from 'react'
import useSimulatedSettings from '@/ui/templates/AppTemplate.simulations.ts'

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

const statusPanelArgs = {
  address: '00:11:22:33:44:55',
  configFailed: false,
  onResumeStatusPolling: fn(),
  onRetryConfig: fn(),
  status: 'active' as const,
  statusPollFailed: false,
}

// Every organism on the real `Settings` page is prop-driven, so the story supplies simulated
// Round trips rather than `fn()` no-ops — see `AppTemplate.simulations.ts` for what each one
// Reproduces, and which current bugs it preserves on purpose.
const InteractiveSettingsSlot = (): React.JSX.Element => {
  const { channelOverrides, daemonControls, securityLevel } = useSimulatedSettings()
  return (
    <Settings
      appearance={appearanceArgs}
      channelOverrides={channelOverrides}
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
