import type { Meta, StoryObj } from '@storybook/react-vite'
import Avatar from '@/ui/atoms/Avatar.tsx'

const meta = {
  component: Avatar,
  title: 'atoms/Avatar',
} satisfies Meta<typeof Avatar>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
