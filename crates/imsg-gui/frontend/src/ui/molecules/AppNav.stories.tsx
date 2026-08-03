import type { Meta, StoryObj } from '@storybook/react-vite'
import AppNav from '@/ui/molecules/AppNav.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    activeScreen: 'messages',
    onSelectScreen: fn(),
  },
  component: AppNav,
  title: 'molecules/AppNav',
} satisfies Meta<typeof AppNav>

export default meta
type Story = StoryObj<typeof meta>

export const MessagesActive: Story = {}

export const SettingsActive: Story = {
  args: { activeScreen: 'settings' },
}
