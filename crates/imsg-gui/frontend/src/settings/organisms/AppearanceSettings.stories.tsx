import type { Meta, StoryObj } from '@storybook/react-vite'
import AppearanceSettings from '@/settings/organisms/AppearanceSettings.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'
import { useCallback } from 'react'

const meta = {
  args: {
    onPreferenceChange: fn(),
    preference: 'system',
  },
  component: AppearanceSettings,
  title: 'organisms/AppearanceSettings',
} satisfies Meta<typeof AppearanceSettings>

export default meta
type Story = StoryObj<typeof meta>

export const System: Story = {}

export const Light: Story = {
  args: { preference: 'light' },
}

export const Dark: Story = {
  args: { preference: 'dark' },
}

type AppearanceSettingsProps = React.ComponentProps<typeof AppearanceSettings>

// `onPreferenceChange: fn()` alone only lets Storybook's Actions panel log the click — it never
// Feeds back into `preference`, so the control would look dead. `useArgs` closes that loop, same
// Pattern as `AppTemplate.stories.tsx`'s `Interactive` story.
const InteractiveAppearanceSettings = (): React.JSX.Element => {
  const [args, updateArgs] = useArgs<AppearanceSettingsProps>()

  const handlePreferenceChange: AppearanceSettingsProps['onPreferenceChange'] = useCallback(
    (preference) => {
      args.onPreferenceChange(preference)
      updateArgs({ preference })
    },
    [args, updateArgs],
  )

  return <AppearanceSettings onPreferenceChange={handlePreferenceChange} preference={args.preference} />
}

export const Interactive: Story = {
  render: InteractiveAppearanceSettings,
}
