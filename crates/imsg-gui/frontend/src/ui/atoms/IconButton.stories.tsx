import type { Meta, StoryObj } from '@storybook/react-vite'
import IconButton from '@/ui/atoms/IconButton.tsx'
import { PanelLeftClose } from 'lucide-react'
import { fn } from 'storybook/test'

const meta = {
  args: {
    icon: PanelLeftClose,
    label: 'Collapse panel',
    onClick: fn(),
  },
  component: IconButton,
  title: 'atoms/IconButton',
} satisfies Meta<typeof IconButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Disabled: Story = {
  args: {
    disabled: true,
  },
}
