import type { Meta, StoryObj } from '@storybook/react-vite'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import CollapsedRail from '@/ui/molecules/CollapsedRail.tsx'
import CollapsiblePane from '@/ui/molecules/CollapsiblePane.tsx'
import PaneHeader from '@/ui/molecules/PaneHeader.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    children: <PaneHeader collapseIcon={PanelLeftClose} collapseLabel="Collapse panel" onCollapse={fn()} title="Conversations" />,
    collapsed: false,
    collapsedWidth: 48,
    contentClassName: 'flex flex-col gap-4 p-4',
    expandedWidth: 288,
    rail: <CollapsedRail expandIcon={PanelLeftOpen} expandLabel="Expand panel" onExpand={fn()} />,
    shellClassName: 'flex shrink-0 flex-col border border-line',
  },
  component: CollapsiblePane,
  decorators: [
    (Story) => (
      <div className="h-40">
        <Story />
      </div>
    ),
  ],
  title: 'molecules/CollapsiblePane',
} satisfies Meta<typeof CollapsiblePane>

export default meta
type Story = StoryObj<typeof meta>

export const Expanded: Story = {}

export const Collapsed: Story = {
  args: { collapsed: true },
}
