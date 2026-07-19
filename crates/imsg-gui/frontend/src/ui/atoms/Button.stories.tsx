import type { Meta, StoryObj } from '@storybook/react-vite'
import Button from '@/ui/atoms/Button.tsx'
import { CirclePlus } from 'lucide-react'
import { fn } from 'storybook/test'

const meta = {
  args: {
    children: 'Retry',
    onClick: fn(),
  },
  component: Button,
  title: 'atoms/Button',
} satisfies Meta<typeof Button>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithIcon: Story = {
  args: {
    icon: CirclePlus,
  },
}

export const Disabled: Story = {
  args: {
    disabled: true,
  },
}

export const Ghost: Story = {
  args: {
    ghost: true,
  },
}
