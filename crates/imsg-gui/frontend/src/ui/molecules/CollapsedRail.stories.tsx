import type { Meta, StoryObj } from '@storybook/react-vite'
import CollapsedRail from '@/ui/molecules/CollapsedRail.tsx'
import { PanelLeftOpen } from 'lucide-react'
import { fn } from 'storybook/test'

const meta = {
  args: {
    expandIcon: PanelLeftOpen,
    expandLabel: 'Expand conversations panel',
    onExpand: fn(),
  },
  component: CollapsedRail,
  decorators: [
    (Story) => (
      <div className="h-40 w-12 border border-line">
        <Story />
      </div>
    ),
  ],
  title: 'molecules/CollapsedRail',
} satisfies Meta<typeof CollapsedRail>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
