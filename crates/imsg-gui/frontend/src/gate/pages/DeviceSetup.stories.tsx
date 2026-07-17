import type { Meta, StoryObj } from '@storybook/react-vite'
import DeviceSetup from '@/gate/pages/DeviceSetup.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onRetryList: fn(),
    onRetryPersist: fn(),
    onRetryResolve: fn(),
    onSelectDevice: fn(),
    state: {
      address: undefined,
      devices: [
        { address: '00:11:22:33:44:55', name: "Ethan's iPhone" },
        { address: 'AA:BB:CC:DD:EE:FF', name: null },
      ],
      errorMessage: undefined,
      mapChannel: undefined,
      pbapChannel: undefined,
      stage: 'picking',
    },
  },
  component: DeviceSetup,
  // CenteredScreen already fills the viewport.
  parameters: { layout: 'fullscreen' },
  title: 'pages/DeviceSetup',
} satisfies Meta<typeof DeviceSetup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
