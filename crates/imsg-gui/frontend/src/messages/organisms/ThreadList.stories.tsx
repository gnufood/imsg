import type { Meta, StoryObj } from '@storybook/react-vite'
import ThreadList from '@/messages/organisms/ThreadList.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
  },
  component: ThreadList,
  title: 'organisms/ThreadList',
} satisfies Meta<typeof ThreadList>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    threads: [
      { address: '00:11:22:33:44:55', contact_name: null, latest_ms: 1_752_700_800_000n, latest_outgoing_status: null, total: 12n, unread: 3n },
      { address: 'AA:BB:CC:DD:EE:FF', contact_name: 'Jane Doe', latest_ms: 1_752_614_400_000n, latest_outgoing_status: 'SentConfirmed', total: 4n, unread: 0n },
    ],
  },
}

export const Empty: Story = {
  args: {
    threads: [],
  },
}
