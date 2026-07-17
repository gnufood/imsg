import type { Meta, StoryObj } from '@storybook/react-vite'
import MessageComposer from '@/ui/molecules/MessageComposer.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    error: undefined,
    onSend: fn(),
    pending: false,
  },
  component: MessageComposer,
  title: 'molecules/MessageComposer',
} satisfies Meta<typeof MessageComposer>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Pending: Story = {
  args: { pending: true },
}

export const Failed: Story = {
  args: { error: "Couldn't send this message." },
}
