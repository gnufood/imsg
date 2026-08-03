import type { Meta, StoryObj } from '@storybook/react-vite'
import Messages from '@/messages/pages/Messages.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    conversationMessages: [
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
    ],
    conversationPollFailed: false,
    deleteConfirmOpen: false,
    deleteError: undefined,
    deleting: false,
    onCancelDelete: fn(),
    onConfirmDelete: fn(),
    onRefreshContacts: fn(),
    onRequestDelete: fn(),
    onResumeConversationPolling: fn(),
    onResumeThreadsPolling: fn(),
    onSelectThread: fn(),
    onSendMessage: fn(),
    refreshingContacts: false,
    selectedAddress: '00:11:22:33:44:55',
    sendError: undefined,
    sendPending: false,
    threads: [
      { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
      { address: 'AA:BB:CC:DD:EE:FF', contact_name: 'Jane Doe', latest_ms: 1_752_614_400_000n, latest_outgoing_status: 'SentConfirmed', total: 4n, unread: 0n },
    ],
    threadsPollFailed: false,
  },
  component: Messages,
  // CenteredScreen (loading/error) and the split-pane layout (ready) both already fill the
  // Viewport.
  parameters: { layout: 'fullscreen' },
  title: 'pages/Messages',
} satisfies Meta<typeof Messages>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const DeleteConfirming: Story = {
  args: { deleteConfirmOpen: true },
}

export const DeleteFailed: Story = {
  args: { deleteConfirmOpen: true, deleteError: "Couldn't reach the device." },
}
