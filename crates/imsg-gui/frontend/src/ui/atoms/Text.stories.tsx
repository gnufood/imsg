import type { Meta, StoryObj } from '@storybook/react-vite'
import Text from './Text.tsx'

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
