import type { Meta, StoryObj } from '@storybook/react-vite'
import ConversationView from '@/messages/organisms/ConversationView.tsx'

const meta = {
  component: ConversationView,
  title: 'organisms/ConversationView',
} satisfies Meta<typeof ConversationView>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
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
  },
}

export const Empty: Story = {
  args: {
    messages: [],
  },
}
