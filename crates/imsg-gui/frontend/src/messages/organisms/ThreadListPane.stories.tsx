import type { Meta, StoryObj } from '@storybook/react-vite'
import ThreadListPane from '@/messages/organisms/ThreadListPane.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
    onToggle: fn(),
    threads: [
      { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
      { address: 'AA:BB:CC:DD:EE:FF', contact_name: 'Jane Doe', latest_ms: 1_752_614_400_000n, latest_outgoing_status: 'SentConfirmed', total: 4n, unread: 0n },
    ],
  },
  component: ThreadListPane,
  parameters: { layout: 'fullscreen' },
  title: 'organisms/ThreadListPane',
} satisfies Meta<typeof ThreadListPane>

export default meta
type Story = StoryObj<typeof meta>

export const Expanded: Story = {
  args: { collapsed: false },
}

export const Collapsed: Story = {
  args: { collapsed: true },
}
