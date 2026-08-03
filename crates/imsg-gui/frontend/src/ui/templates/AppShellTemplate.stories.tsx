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
