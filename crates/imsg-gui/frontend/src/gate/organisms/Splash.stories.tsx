import type { Meta, StoryObj } from '@storybook/react-vite'
import Splash from '@/gate/organisms/Splash.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onDone: fn(),
  },
  component: Splash,
  // CenteredScreen already fills the viewport — avoid double-centering it inside the canvas.
  parameters: { layout: 'fullscreen' },
  title: 'organisms/Splash',
} satisfies Meta<typeof Splash>

export default meta
type Story = StoryObj<typeof meta>

// Backing check never resolves — splash holds past both the floor and the cap.
export const Pending: Story = {
  args: { ready: false },
}

// Backing check already resolved — fades out once the floor timer clears (~3.8s).
export const Ready: Story = {
  args: { ready: true },
}
