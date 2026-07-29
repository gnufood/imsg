import type { Meta, StoryObj } from '@storybook/react-vite'
import DaemonControls from '@/settings/organisms/DaemonControls.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    installError: undefined,
    installing: false,
    onInstall: fn(),
    onRestart: fn(),
    onResumeServiceStatusPolling: fn(),
    onStop: fn(),
    onUninstall: fn(),
    restartError: undefined,
    restarting: false,
    serviceStatusPollFailed: false,
    stopError: undefined,
    stopping: false,
    systemInstalled: false,
    uninstallError: undefined,
    uninstalling: false,
    userInstalled: false,
  },
  component: DaemonControls,
  title: 'organisms/DaemonControls',
} satisfies Meta<typeof DaemonControls>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const UserInstalled: Story = {
  args: { systemInstalled: false, userInstalled: true },
}

// A system daemon installed from the CLI: reported here, with no action offered on its own row
// And the user row's Install gated, since the two would contend.
export const SystemInstalled: Story = {
  args: { systemInstalled: true, userInstalled: false },
}

// Both installed — already contending. Uninstall stays enabled so the user can resolve it.
export const BothInstalled: Story = {
  args: { systemInstalled: true, userInstalled: true },
}

export const Checking: Story = {
  args: { systemInstalled: undefined, userInstalled: undefined },
}

export const InstallFailed: Story = {
  args: { installError: 'Permission denied registering the service.' },
}

export const ServiceStatusUnavailable: Story = {
  args: { serviceStatusPollFailed: true },
}
