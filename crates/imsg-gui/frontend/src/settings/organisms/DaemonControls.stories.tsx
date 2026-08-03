import type { Meta, StoryObj } from '@storybook/react-vite'
import DaemonControls from '@/settings/organisms/DaemonControls.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    installError: undefined,
    installing: false,
    onInstall: fn(),
    onRestart: fn(),
    onStop: fn(),
    onUninstall: fn(),
    restartError: undefined,
    restarting: false,
    stopError: undefined,
    stopping: false,
    uninstallError: undefined,
    uninstalling: false,
  },
  component: DaemonControls,
  title: 'organisms/DaemonControls',
} satisfies Meta<typeof DaemonControls>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Stopping: Story = {
  args: { stopping: true },
}

export const StopFailed: Story = {
  args: { stopError: "Couldn't reach the daemon." },
}

export const InstallFailed: Story = {
  args: { installError: 'Permission denied registering the service.' },
}
