import type { Meta, StoryObj } from '@storybook/react-vite'
import CompactButton from '@/ui/atoms/CompactButton.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    children: 'Cancel',
    disabled: false,
    ghost: false,
    onClick: fn(),
  },
  component: CompactButton,
  title: 'atoms/CompactButton',
} satisfies Meta<typeof CompactButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Ghost: Story = {
  args: { children: 'Detect from device', ghost: true },
}

export const Disabled: Story = {
  args: { disabled: true },
}
