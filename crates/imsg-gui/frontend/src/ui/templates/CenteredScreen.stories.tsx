import type { Meta, StoryObj } from '@storybook/react-vite'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import Text from '@/ui/atoms/Text.tsx'

const meta = {
  args: {
    children: <Text tone="muted">Content fixture</Text>,
  },
  component: CenteredScreen,
  parameters: { layout: 'fullscreen' },
  title: 'templates/CenteredScreen',
} satisfies Meta<typeof CenteredScreen>

export default meta
type Story = StoryObj<typeof meta>

// What the gate screens get: nothing above them, so it fills the window.
export const Viewport: Story = {
  args: { fill: 'viewport' },
}

// What screens nested under `AppShellTemplate`'s nav get. The decorator supplies the bounded box
// That slot provides — `h-full` has nothing to resolve against otherwise, which is precisely the
// Contract this prop makes visible.
export const Parent: Story = {
  args: { fill: 'parent' },
  decorators: [
    (Story) => (
      <div className="h-40 border border-line">
        <Story />
      </div>
    ),
  ],
}
