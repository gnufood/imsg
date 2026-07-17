import type { Meta, StoryObj } from '@storybook/react-vite'
import DeviceList from '@/gate/organisms/DeviceList.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
  },
  component: DeviceList,
  title: 'organisms/DeviceList',
} satisfies Meta<typeof DeviceList>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    devices: [
      { address: '00:11:22:33:44:55', name: "Ethan's iPhone" },
      { address: 'AA:BB:CC:DD:EE:FF', name: null },
      { address: '12:34:56:78:9A:BC', name: 'AirPods Pro' },
    ],
  },
}

export const Empty: Story = {
  args: {
    devices: [],
  },
}
