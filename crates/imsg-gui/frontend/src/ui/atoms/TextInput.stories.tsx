import type { Meta, StoryObj } from '@storybook/react-vite'
import TextInput from '@/ui/atoms/TextInput.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onChange: fn(),
    placeholder: 'Message',
    value: '',
  },
  component: TextInput,
  title: 'atoms/TextInput',
} satisfies Meta<typeof TextInput>

export default meta
type Story = StoryObj<typeof meta>

export const Empty: Story = {}

export const Filled: Story = {
  args: {
    value: 'Hey, are we still on for tonight?',
  },
}

export const Disabled: Story = {
  args: {
    disabled: true,
    value: 'Sending…',
  },
}
