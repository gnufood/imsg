import type { Meta, StoryObj } from '@storybook/react-vite'
import ChannelOverridesForm from '@/settings/organisms/ChannelOverridesForm.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    error: undefined,
    mapDraft: '20',
    onMapDraftChange: fn(),
    onPbapDraftChange: fn(),
    onSave: fn(),
    pbapDraft: '21',
    saving: false,
  },
  component: ChannelOverridesForm,
  title: 'organisms/ChannelOverridesForm',
} satisfies Meta<typeof ChannelOverridesForm>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Saving: Story = {
  args: { saving: true },
}

export const Failed: Story = {
  args: { error: 'Channels must be numbers.' },
}
