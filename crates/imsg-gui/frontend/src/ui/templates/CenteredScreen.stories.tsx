import type { Meta, StoryObj } from '@storybook/react-vite'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import Text from '@/ui/atoms/Text.tsx'

const meta = {
  args: {
    children: <Text tone="muted">Content fixture</Text>,
  },
  component: CenteredScreen,
  // Fills the viewport (min-h-screen) — same as every template that wraps it.
  parameters: { layout: 'fullscreen' },
  title: 'templates/CenteredScreen',
} satisfies Meta<typeof CenteredScreen>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
