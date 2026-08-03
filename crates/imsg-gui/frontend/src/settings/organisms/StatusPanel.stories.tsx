import type { Meta, StoryObj } from '@storybook/react-vite'
import StatusPanel from '@/settings/organisms/StatusPanel.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    address: '00:11:22:33:44:55',
    configFailed: false,
    onResumeStatusPolling: fn(),
    onRetryConfig: fn(),
    status: 'active',
    statusPollFailed: false,
  },
  component: StatusPanel,
  title: 'organisms/StatusPanel',
} satisfies Meta<typeof StatusPanel>

export default meta
type Story = StoryObj<typeof meta>

export const Active: Story = {}

export const Connecting: Story = {
  args: { status: 'connecting' },
}

export const NoDaemon: Story = {
  args: { status: null },
}

export const Failed: Story = {
  args: { status: 'failed' },
}

export const LoadingConfig: Story = {
  args: { address: undefined },
}

export const ConfigFailed: Story = {
  args: { configFailed: true },
}

export const StatusPollFailed: Story = {
  args: { statusPollFailed: true },
}
