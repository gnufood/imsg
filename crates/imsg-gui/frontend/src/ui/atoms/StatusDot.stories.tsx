import type { Meta, StoryObj } from '@storybook/react-vite'
import StatusDot from '@/ui/atoms/StatusDot.tsx'

const meta = {
  args: {
    tone: 'active',
  },
  component: StatusDot,
  title: 'atoms/StatusDot',
} satisfies Meta<typeof StatusDot>

export default meta
type Story = StoryObj<typeof meta>

export const Active: Story = {}

export const Pending: Story = {
  args: { tone: 'pending' },
}

export const Idle: Story = {
  args: { tone: 'idle' },
}

export const Error: Story = {
  args: { tone: 'error' },
}
