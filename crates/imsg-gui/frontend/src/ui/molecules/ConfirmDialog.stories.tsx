import type { Meta, StoryObj } from '@storybook/react-vite'
import ConfirmDialog from '@/ui/molecules/ConfirmDialog.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    confirmLabel: 'Delete',
    error: undefined,
    message: "Delete all messages with 00:11:22:33:44:55? This can't be undone.",
    onCancel: fn(),
    onConfirm: fn(),
    title: 'Delete conversation?',
  },
  component: ConfirmDialog,
  parameters: { layout: 'fullscreen' },
  title: 'molecules/ConfirmDialog',
} satisfies Meta<typeof ConfirmDialog>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Pending: Story = {
  args: { pending: true },
}

export const Failed: Story = {
  args: { error: "Couldn't reach the device." },
}
