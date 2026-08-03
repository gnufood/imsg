import type { Meta, StoryObj } from '@storybook/react-vite'
import Checkbox from '@/ui/atoms/Checkbox.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    checked: false,
    label: 'Install as a system service',
    onChange: fn(),
  },
  component: Checkbox,
  title: 'atoms/Checkbox',
} satisfies Meta<typeof Checkbox>

export default meta
type Story = StoryObj<typeof meta>

export const Unchecked: Story = {}

export const Checked: Story = {
  args: { checked: true },
}

export const Disabled: Story = {
  args: { disabled: true },
}
