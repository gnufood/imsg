import type { Meta, StoryObj } from '@storybook/react-vite'
import SettingsTemplate from '@/settings/templates/SettingsTemplate.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    appearance: {
      onPreferenceChange: fn(),
      preference: 'system',
    },
    channelOverrides: {
      detectError: undefined,
      detecting: false,
      editorOpen: false,
      mapChannel: 8,
      mapDraft: '8',
      onCancel: fn(),
      onDetect: fn(),
      onMapDraftChange: fn(),
      onPbapDraftChange: fn(),
      onRequestEdit: fn(),
      onSave: fn(),
      pbapChannel: 12,
      pbapDraft: '12',
      saveError: undefined,
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
    securityLevel: {
      committedLevel: 'Medium',
      draft: 'Medium',
      onCancel: fn(),
      onDraftChange: fn(),
      onSave: fn(),
      saveError: undefined,
      saving: false,
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
  component: SettingsTemplate,
  // Fills its parent (`AppShellTemplate`'s content slot), not the viewport — the decorator stands
  // In for that slot, and `h-full` has nothing to resolve against without it.
  decorators: [
    (Story) => (
      <div className="h-screen">
        <Story />
      </div>
    ),
  ],
  parameters: { layout: 'fullscreen' },
  title: 'templates/SettingsTemplate',
} satisfies Meta<typeof SettingsTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

// The only place this screen's scroll region is observable. 555px = `tauri.conf.json`'s
// `minHeight: 600` less the 45px nav (`size-7` button + `py-2` + `border-b`) — the smallest
// Content slot the window can produce, where five sections genuinely overflow. Scrolling here
// Must move the content only; the nav is outside this box and never moves.
export const MinimumWindowHeight: Story = {
  decorators: [
    (Story) => (
      <div className="h-[555px] border-b border-line">
        <Story />
      </div>
    ),
  ],
}
