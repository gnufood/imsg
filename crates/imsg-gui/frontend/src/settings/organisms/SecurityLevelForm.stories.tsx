import type { Meta, StoryObj } from '@storybook/react-vite'
import SecurityLevelForm from '@/settings/organisms/SecurityLevelForm.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    committedLevel: 'Medium',
    draft: 'Medium',
    onCancel: fn(),
    onDraftChange: fn(),
    onSave: fn(),
    saveError: undefined,
    saving: false,
  },
  component: SecurityLevelForm,
  title: 'organisms/SecurityLevelForm',
} satisfies Meta<typeof SecurityLevelForm>

export default meta
type Story = StoryObj<typeof meta>

// Default renders the summary row only — the editor lives behind the "Change…" toggle, which is
// Local `isEditing` state the component owns itself (not an arg), same reason
// `ChannelOverridesForm.stories.tsx` has no dedicated editor-open story either. Click through in
// The canvas to reach the editor.
export const Default: Story = {}

export const Unset: Story = {
  args: { committedLevel: null },
}

export const Loading: Story = {
  args: { committedLevel: undefined },
}
