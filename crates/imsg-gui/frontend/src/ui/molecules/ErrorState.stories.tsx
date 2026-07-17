import type { Meta, StoryObj } from '@storybook/react-vite'
import ErrorState from './ErrorState.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    message: 'Bluetooth adapter is off.',
    onRetry: fn(),
  },
  component: ErrorState,
  title: 'molecules/ErrorState',
} satisfies Meta<typeof ErrorState>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
