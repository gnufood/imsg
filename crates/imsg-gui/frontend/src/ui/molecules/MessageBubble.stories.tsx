import type { Meta, StoryObj } from '@storybook/react-vite'
import MessageBubble from '@/ui/molecules/MessageBubble.tsx'

const meta = {
  component: MessageBubble,
  // <li> requires a list ancestor for correct a11y semantics.
  decorators: [
    (Story) => (
      <ul className="flex w-full max-w-sm flex-col gap-2">
        <Story />
      </ul>
    ),
  ],
  title: 'molecules/MessageBubble',
} satisfies Meta<typeof MessageBubble>

export default meta
type Story = StoryObj<typeof meta>

export const Received: Story = {
  args: {
    message: {
      address: '00:11:22:33:44:55',
      direction: 'Received',
      folder: 'telecom/msg/inbox',
      handle: '1001',
      outgoing_status: null,
      read: true,
      text: "Hey, are we still on for tonight?",
      timestamp_ms: 1_752_700_800_000n,
    },
  },
}

export const Sent: Story = {
  args: {
    message: {
      address: '00:11:22:33:44:55',
      direction: 'Sent',
      folder: 'telecom/msg/sent',
      handle: '1002',
      outgoing_status: 'SentConfirmed',
      read: true,
      text: 'Yep, see you at 7.',
      timestamp_ms: 1_752_700_860_000n,
    },
  },
}

export const SendFailed: Story = {
  args: {
    message: {
      address: '00:11:22:33:44:55',
      direction: 'Sent',
      folder: 'telecom/msg/outbox',
      handle: '1003',
      outgoing_status: 'FailedPermanent',
      read: true,
      text: 'Running 10 minutes late.',
      timestamp_ms: 1_752_700_920_000n,
    },
  },
}
