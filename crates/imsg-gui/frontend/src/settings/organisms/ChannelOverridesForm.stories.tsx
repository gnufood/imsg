import type { Meta, StoryObj } from '@storybook/react-vite'
import ChannelOverridesForm from '@/settings/organisms/ChannelOverridesForm.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    detectError: undefined,
    detecting: false,
    mapChannel: 8,
    mapDraft: '8',
    onCancel: fn(),
    onDetect: fn(),
    onMapDraftChange: fn(),
    onPbapDraftChange: fn(),
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

// Default renders the summary rows only — Saving/SaveFailed/Detecting/DetectFailed all live
// Behind the "Change manually…" toggle, which is local `isEditing` state the component owns
// Itself (not an arg), same reason `DaemonControls.stories.tsx` has no dedicated
// Confirm-dialog-open story either. Click through in the canvas to reach the editor.
export const Default: Story = {}

export const Loading: Story = {
  args: { mapChannel: undefined, pbapChannel: undefined },
}
