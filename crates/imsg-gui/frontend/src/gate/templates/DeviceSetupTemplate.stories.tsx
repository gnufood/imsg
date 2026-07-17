import type { Meta, StoryObj } from '@storybook/react-vite'
import DeviceSetupTemplate from '@/gate/templates/DeviceSetupTemplate.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onRetryList: fn(),
    onRetryPersist: fn(),
    onRetryResolve: fn(),
    onSelectDevice: fn(),
    state: {
      address: undefined,
      devices: [],
      errorMessage: undefined,
      mapChannel: undefined,
      pbapChannel: undefined,
      stage: 'listing',
    },
  },
  component: DeviceSetupTemplate,
  // CenteredScreen already fills the viewport.
  parameters: { layout: 'fullscreen' },
  title: 'templates/DeviceSetupTemplate',
} satisfies Meta<typeof DeviceSetupTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const Listing: Story = {}

export const ListError: Story = {
  args: { state: { ...meta.args.state, errorMessage: "Couldn't list paired devices.", stage: 'listError' } },
}

export const Picking: Story = {
  args: {
    state: {
      ...meta.args.state,
      devices: [
        { address: '00:11:22:33:44:55', name: "Ethan's iPhone" },
        { address: 'AA:BB:CC:DD:EE:FF', name: null },
      ],
      stage: 'picking',
    },
  },
}

export const Resolving: Story = {
  args: { state: { ...meta.args.state, address: '00:11:22:33:44:55', stage: 'resolving' } },
}

export const ResolveError: Story = {
  args: {
    state: { ...meta.args.state, errorMessage: "Couldn't check messaging support.", stage: 'resolveError' },
  },
}

export const Unsupported: Story = {
  args: { state: { ...meta.args.state, stage: 'unsupported' } },
}

export const Persisting: Story = {
  args: { state: { ...meta.args.state, address: '00:11:22:33:44:55', mapChannel: 3, pbapChannel: 5, stage: 'persisting' } },
}

export const PersistError: Story = {
  args: { state: { ...meta.args.state, errorMessage: "Couldn't save device.", stage: 'persistError' } },
}
