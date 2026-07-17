import type { Meta, StoryObj } from '@storybook/react-vite'
import LoadingState from './LoadingState.tsx'

const meta = {
  args: {
    message: 'Starting the daemon…',
  },
  component: LoadingState,
  title: 'molecules/LoadingState',
} satisfies Meta<typeof LoadingState>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
