import type { Meta, StoryObj } from '@storybook/react-vite'
import DeviceListItem from '@/ui/molecules/DeviceListItem.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
  },
  component: DeviceListItem,
  // <li> requires a list ancestor for correct a11y semantics.
  decorators: [
    (Story) => (
      <ul>
        <Story />
      </ul>
    ),
  ],
  title: 'molecules/DeviceListItem',
} satisfies Meta<typeof DeviceListItem>

export default meta
type Story = StoryObj<typeof meta>

export const Named: Story = {
  args: {
    device: { address: '00:11:22:33:44:55', name: "Ethan's iPhone" },
  },
}

export const Unnamed: Story = {
  args: {
    device: { address: '00:11:22:33:44:55', name: null },
  },
}
