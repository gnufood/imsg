import type { Meta, StoryObj } from '@storybook/react-vite'
import SecurityLevelForm from '@/settings/organisms/SecurityLevelForm.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'
import { useCallback } from 'react'

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

// Draft equals committed — segments render, Apply/Cancel stay hidden since there's nothing to
// Persist yet.
export const Default: Story = {}

// Draft diverges from committed — the state that surfaces Apply/Cancel.
export const Dirty: Story = {
  args: { draft: 'High' },
}

export const SaveFailed: Story = {
  args: { draft: 'High', saveError: 'Broker rejected the change: not connected.' },
}

export const Unset: Story = {
  args: { committedLevel: null, draft: 'Sdp' },
}

export const Loading: Story = {
  args: { committedLevel: undefined },
}

type SecurityLevelFormProps = React.ComponentProps<typeof SecurityLevelForm>

// `onDraftChange: fn()` alone only lets Storybook's Actions panel log the click — it never feeds
// Back into `draft`, so the segments would look dead and Apply/Cancel would never appear. `useArgs`
// Closes that loop, same pattern as `SegmentedControl.stories.tsx`'s own `Interactive` story.
const InteractiveSecurityLevelForm = (): React.JSX.Element => {
  const [args, updateArgs] = useArgs<SecurityLevelFormProps>()

  const handleDraftChange: SecurityLevelFormProps['onDraftChange'] = useCallback(
    (draft) => {
      args.onDraftChange(draft)
      updateArgs({ draft })
    },
    [args, updateArgs],
  )

  const handleCancel = args.onCancel
  const handleSave = args.onSave

  return (
    <SecurityLevelForm
      committedLevel={args.committedLevel}
      draft={args.draft}
      onCancel={handleCancel}
      onDraftChange={handleDraftChange}
      onSave={handleSave}
      saveError={args.saveError}
      saving={args.saving}
    />
  )
}

export const Interactive: Story = {
  render: InteractiveSecurityLevelForm,
}
