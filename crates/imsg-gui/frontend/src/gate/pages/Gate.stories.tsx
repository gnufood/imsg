import type { Meta, StoryObj } from '@storybook/react-vite'
import Gate from './Gate.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    deviceSetupSlot: <p className="text-sm text-muted">Device setup fixture</p>,
    onProceed: fn(),
    onReady: fn(),
    onResumePolling: fn(),
    pollFailed: false,
    status: 'StartingDaemon',
  },
  component: Gate,
  // CenteredScreen (used by most branches) already fills the viewport.
  parameters: { layout: 'fullscreen' },
  title: 'pages/Gate',
} satisfies Meta<typeof Gate>

export default meta
type Story = StoryObj<typeof meta>

// `splashDone` is Gate's own local state (see GUI_ATOMIC_DESIGN.md) — Splash plays out
// Normally in this story before falling through to `status`.
export const Default: Story = {}

export const BackendUnreachable: Story = {
  args: { pollFailed: true },
}
