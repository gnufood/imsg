import type { Meta, StoryObj } from '@storybook/react-vite'
import Text from '@/ui/atoms/Text.tsx'

const meta = {
  args: {
    children: 'Looking for paired devices…',
  },
  component: Text,
  title: 'atoms/Text',
} satisfies Meta<typeof Text>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Muted: Story = {
  args: {
    tone: 'muted',
  },
}

export const Small: Story = {
  args: {
    size: 'xs',
    tone: 'muted',
  },
}

export const Surface: Story = {
  args: {
    tone: 'surface',
  },
  // `surface` is near-white — needs a dark backdrop to be visible (its actual use case is
  // Text over a `bg-ink` sent-message bubble, not the canvas's own `surface` background).
  decorators: [
    (Story) => (
      <div className="bg-ink p-4">
        <Story />
      </div>
    ),
  ],
}
