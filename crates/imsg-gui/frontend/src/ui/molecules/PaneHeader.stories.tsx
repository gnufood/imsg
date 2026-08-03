import type { Meta, StoryObj } from '@storybook/react-vite'
import PaneHeader from '@/ui/molecules/PaneHeader.tsx'
import { PanelLeftClose } from 'lucide-react'
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
