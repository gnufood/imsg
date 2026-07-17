import type { Meta, StoryObj } from '@storybook/react-vite'
import MessagesTemplate from '@/messages/templates/MessagesTemplate.tsx'
import { fn } from 'storybook/test'

const THREADS = [
  { address: '00:11:22:33:44:55', latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
  { address: 'AA:BB:CC:DD:EE:FF', latest_ms: 1_752_614_400_000n, latest_outgoing_status: 'SentConfirmed', total: 4n, unread: 0n },
] as const

const MESSAGES = [
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
] as const

const meta = {
  args: {
    conversationMessages: undefined,
    conversationPollFailed: false,
    onResumeConversationPolling: fn(),
    onResumeThreadsPolling: fn(),
    onSelectThread: fn(),
    onSendMessage: fn(),
    selectedAddress: undefined,
    sendError: undefined,
    sendPending: false,
    threads: undefined,
    threadsPollFailed: false,
  },
  component: MessagesTemplate,
  // CenteredScreen (loading/error) and the split-pane layout (ready) both already fill the
  // Viewport.
  parameters: { layout: 'fullscreen' },
  title: 'templates/MessagesTemplate',
} satisfies Meta<typeof MessagesTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const LoadingThreads: Story = {}

export const ThreadsFailed: Story = {
  args: { threadsPollFailed: true },
}

export const NoThreadSelected: Story = {
  args: { threads: [...THREADS] },
}

export const LoadingConversation: Story = {
  args: { selectedAddress: THREADS[0].address, threads: [...THREADS] },
}

export const ConversationFailed: Story = {
  args: { conversationPollFailed: true, selectedAddress: THREADS[0].address, threads: [...THREADS] },
}

export const ConversationReady: Story = {
  args: { conversationMessages: [...MESSAGES], selectedAddress: THREADS[0].address, threads: [...THREADS] },
}
