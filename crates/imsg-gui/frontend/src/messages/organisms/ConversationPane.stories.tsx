import type { Meta, StoryObj } from '@storybook/react-vite'
import ConversationPane from '@/messages/organisms/ConversationPane.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    deleting: false,
    messages: [
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
    onDelete: fn(),
    onResumePolling: fn(),
    onSendMessage: fn(),
    pollFailed: false,
    selectedAddress: '00:11:22:33:44:55',
    sendError: undefined,
    sendPending: false,
  },
  component: ConversationPane,
  parameters: { layout: 'fullscreen' },
  title: 'organisms/ConversationPane',
} satisfies Meta<typeof ConversationPane>

export default meta
type Story = StoryObj<typeof meta>

export const Ready: Story = {}

export const Empty: Story = {
  args: { messages: [], selectedAddress: undefined },
}

export const Loading: Story = {
  args: { messages: undefined },
}

export const Failed: Story = {
  args: { pollFailed: true },
}

export const Deleting: Story = {
  args: { deleting: true },
}
