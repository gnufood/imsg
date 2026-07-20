import type { Meta, StoryObj } from '@storybook/react-vite'
import SummaryRow from '@/ui/molecules/SummaryRow.tsx'

const meta = {
  args: {
    mark: 'MAP',
    value: '8',
    valueLabel: 'Channel',
  },
  component: SummaryRow,
  title: 'molecules/SummaryRow',
} satisfies Meta<typeof SummaryRow>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Pbap: Story = {
  args: {
    mark: 'PBAP',
    value: '12',
  },
}
