import type { Meta, StoryObj } from '@storybook/react-vite'
import ChannelOverridesForm from '@/settings/organisms/ChannelOverridesForm.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    detectError: undefined,
    detecting: false,
    editorOpen: false,
    mapChannel: 8,
    mapDraft: '8',
    onCancel: fn(),
    onDetect: fn(),
    onMapDraftChange: fn(),
    onPbapDraftChange: fn(),
    onRequestEdit: fn(),
    onSave: fn(),
    pbapChannel: 12,
    pbapDraft: '12',
    saveError: undefined,
    saving: false,
  },
  component: ChannelOverridesForm,
  title: 'organisms/ChannelOverridesForm',
} satisfies Meta<typeof ChannelOverridesForm>

export default meta
type Story = StoryObj<typeof meta>

// Collapsed to the summary rows plus the "Change manually…" button.
export const Default: Story = {}

export const Loading: Story = {
  args: { mapChannel: undefined, pbapChannel: undefined },
}

// Every editor state below is reachable from args because `editorOpen` is owned by
// `use-channel-overrides.ts`, not by this component — the tier placement is what makes them
// Renderable here at all (see internal/GUI_ATOMIC_DESIGN.md's closing rule).
export const Editing: Story = {
  args: { editorOpen: true },
}

// Both selects and both buttons disable while a write is in flight (`pending`).
export const Saving: Story = {
  args: { editorOpen: true, mapDraft: '16', pbapDraft: '19', saving: true },
}

// The editor deliberately stays open on a failed Apply, error in place and drafts intact.
export const SaveFailed: Story = {
  args: { editorOpen: true, mapDraft: '16', pbapDraft: '19', saveError: '16 is not in [1, 30]' },
}

export const Detecting: Story = {
  args: { detecting: true, editorOpen: true },
}

export const DetectFailed: Story = {
  args: { detectError: 'sdp: PBAP query to EC:0D:51:C7:58:64 failed: connect failed', editorOpen: true },
}
