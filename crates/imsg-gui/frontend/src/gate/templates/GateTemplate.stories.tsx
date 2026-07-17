import type { Meta, StoryObj } from '@storybook/react-vite'
import GateTemplate from '@/gate/templates/GateTemplate.tsx'
import Text from '@/ui/atoms/Text.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    deviceSetupSlot: <Text tone="muted">Device setup fixture</Text>,
    onProceed: fn(),
    onResumePolling: fn(),
    onSplashDone: fn(),
    pollFailed: false,
    splashDone: true,
    status: 'Initializing',
  },
  component: GateTemplate,
  // CenteredScreen (used by most branches) already fills the viewport.
  parameters: { layout: 'fullscreen' },
  title: 'templates/GateTemplate',
} satisfies Meta<typeof GateTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const Loading: Story = {}

export const Splash: Story = {
  args: { splashDone: false },
}

export const BackendUnreachable: Story = {
  args: { pollFailed: true },
}

export const AwaitingDeviceConfig: Story = {
  args: { status: 'AwaitingDeviceConfig' },
}

export const StartingDaemon: Story = {
  args: { status: 'StartingDaemon' },
}

export const Syncing: Story = {
  args: { status: 'Syncing' },
}

export const DaemonFailed: Story = {
  args: { status: { Failed: { message: 'Bluetooth adapter is off.', stage: 'Daemon' } } },
}

export const SyncFailed: Story = {
  args: { status: { Failed: { message: 'Lost connection to the device.', stage: 'Sync' } } },
}
