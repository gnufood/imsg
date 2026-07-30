import type { Meta, StoryObj } from '@storybook/react-vite'
import AppNav from '@/ui/molecules/AppNav.tsx'
import AppShellTemplate from '@/ui/templates/AppShellTemplate.tsx'
import Text from '@/ui/atoms/Text.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    children: (
      <div className="p-6">
        <Text tone="muted">Screen content fixture</Text>
      </div>
    ),
    nav: <AppNav activeScreen="messages" onSelectScreen={fn()} />,
  },
  component: AppShellTemplate,
  parameters: { layout: 'fullscreen' },
  title: 'templates/AppShellTemplate',
} satisfies Meta<typeof AppShellTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

// The outer bound, made visible: a child taller than the slot is clipped here rather than
// Scrolling the document and taking the nav with it. Screens that legitimately overflow (see
// `SettingsTemplate`) own a scroll region instead of relying on this.
export const OverflowingChildIsContained: Story = {
  args: {
    children: (
      <div className="h-[200vh] bg-accent/10 p-6">
        <Text tone="muted">Child twice the viewport height — the nav stays put.</Text>
      </div>
    ),
  },
}
