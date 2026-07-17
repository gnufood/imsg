import type { Meta, StoryObj } from '@storybook/react-vite'
import EmptyState from '@/ui/molecules/EmptyState.tsx'

const meta = {
  args: {
    message: 'No paired devices found.',
  },
  component: EmptyState,
  title: 'molecules/EmptyState',
} satisfies Meta<typeof EmptyState>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
