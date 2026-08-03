import type { Meta, StoryObj } from '@storybook/react-vite'
import ContactSearchBox from '@/ui/molecules/ContactSearchBox.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    active: false,
    error: undefined,
    onClear: fn(),
    onSearch: fn(),
    searching: false,
  },
  component: ContactSearchBox,
  title: 'molecules/ContactSearchBox',
} satisfies Meta<typeof ContactSearchBox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Searching: Story = {
  args: { searching: true },
}

export const Active: Story = {
  args: { active: true },
}

export const Failed: Story = {
  args: { error: 'No contact found for that number.' },
}
