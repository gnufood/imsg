import type { Meta, StoryObj } from '@storybook/react-vite'
import ThreadListItem from '@/ui/molecules/ThreadListItem.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
  },
  component: ThreadListItem,
  // <li> requires a list ancestor for correct a11y semantics.
  decorators: [
    (Story) => (
      <ul>
        <Story />
      </ul>
    ),
  ],
  title: 'molecules/ThreadListItem',
} satisfies Meta<typeof ThreadListItem>

export default meta
type Story = StoryObj<typeof meta>

export const Read: Story = {
  args: {
    thread: { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 0n },
  },
}

export const Unread: Story = {
  args: {
    thread: { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
  },
}

export const WithContactName: Story = {
  args: {
    thread: { address: '00:11:22:33:44:55', contact_name: 'Jane Doe', latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 0n },
  },
}
