import type { Meta, StoryObj } from '@storybook/react-vite'
import ServiceStatusCard from '@/ui/molecules/ServiceStatusCard.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    description: 'Runs when you sign in — no administrator access.',
    disabled: false,
    installed: false,
    onToggle: fn(),
    title: 'User service',
  },
  component: ServiceStatusCard,
  title: 'molecules/ServiceStatusCard',
} satisfies Meta<typeof ServiceStatusCard>

export default meta
type Story = StoryObj<typeof meta>

export const NotInstalled: Story = {}

export const Installed: Story = {
  args: { installed: true },
}

export const Checking: Story = {
  args: { installed: undefined },
}

export const Pending: Story = {
  args: { disabled: true, installed: true },
}
