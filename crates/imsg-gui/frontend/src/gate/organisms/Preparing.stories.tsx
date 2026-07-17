import type { Meta, StoryObj } from '@storybook/react-vite'
import Preparing from './Preparing.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onRetry: fn(),
  },
  component: Preparing,
  // CenteredScreen already fills the viewport — avoid double-centering it inside the canvas.
  parameters: { layout: 'fullscreen' },
  title: 'organisms/Preparing',
} satisfies Meta<typeof Preparing>

export default meta
type Story = StoryObj<typeof meta>

export const StartingDaemon: Story = {
  args: { stage: 'daemon' },
}

export const SyncingMessages: Story = {
  args: { stage: 'sync' },
}

export const DaemonFailed: Story = {
  args: { error: 'Bluetooth adapter is off.', stage: 'daemon' },
}

export const SyncFailed: Story = {
  args: { error: 'Lost connection to the device.', stage: 'sync' },
}
