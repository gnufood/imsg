import type { Meta, StoryObj } from '@storybook/react-vite'
import Settings from '@/settings/pages/Settings.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    appearance: {
      onPreferenceChange: fn(),
      preference: 'system',
    },
    channelOverrides: {
      error: undefined,
      mapDraft: '20',
      onMapDraftChange: fn(),
      onPbapDraftChange: fn(),
      onSave: fn(),
      pbapDraft: '21',
      saving: false,
    },
    daemonControls: {
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
    statusPanel: {
      address: '00:11:22:33:44:55',
      configFailed: false,
      onResumeStatusPolling: fn(),
      onRetryConfig: fn(),
      status: 'active',
      statusPollFailed: false,
    },
  },
  component: Settings,
  parameters: { layout: 'fullscreen' },
  title: 'pages/Settings',
} satisfies Meta<typeof Settings>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
