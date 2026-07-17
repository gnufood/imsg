import type { Meta, StoryObj } from '@storybook/react-vite'
import Badge from '@/ui/atoms/Badge.tsx'
import { MessageSquareDot } from 'lucide-react'

const meta = {
  args: {
    count: 3,
  },
  component: Badge,
  title: 'atoms/Badge',
} satisfies Meta<typeof Badge>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithIcon: Story = {
  args: {
    icon: MessageSquareDot,
  },
}
