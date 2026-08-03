import type { Meta, StoryObj } from '@storybook/react-vite'
import { PanelLeftClose, RefreshCw } from 'lucide-react'
import PaneHeader from '@/ui/molecules/PaneHeader.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    collapseIcon: PanelLeftClose,
    collapseLabel: 'Collapse conversations panel',
    onCollapse: fn(),
    title: 'Conversations',
  },
  component: PaneHeader,
  title: 'molecules/PaneHeader',
} satisfies Meta<typeof PaneHeader>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithSecondaryAction: Story = {
  args: { onSecondaryAction: fn(), secondaryIcon: RefreshCw, secondaryLabel: 'Refresh contacts' },
}

export const SecondaryActionPending: Story = {
  args: { onSecondaryAction: fn(), secondaryIcon: RefreshCw, secondaryLabel: 'Refresh contacts', secondaryPending: true },
}
