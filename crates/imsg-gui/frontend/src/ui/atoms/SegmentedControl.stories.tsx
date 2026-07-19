import type { Meta, StoryObj } from '@storybook/react-vite'
import SegmentedControl from '@/ui/atoms/SegmentedControl.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'
import { useCallback } from 'react'

const OPTIONS = [
  { label: 'Light', value: 'light' },
  { label: 'Dark', value: 'dark' },
  { label: 'System', value: 'system' },
]

const meta = {
  args: {
    disabled: false,
    name: 'theme-preference',
    onChange: fn(),
    options: OPTIONS,
    value: 'system',
  },
  component: SegmentedControl<string>,
  title: 'atoms/SegmentedControl',
} satisfies Meta<typeof SegmentedControl<string>>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const LightSelected: Story = {
  args: { value: 'light' },
}

export const Disabled: Story = {
  args: { disabled: true },
}

type SegmentedControlProps = React.ComponentProps<typeof SegmentedControl<string>>

// `onChange: fn()` alone only lets Storybook's Actions panel log the click — it never feeds back
// Into `value`, so the control would look dead. `useArgs` closes that loop, same pattern as
// `AppTemplate.stories.tsx`'s `Interactive` story.
const InteractiveSegmentedControl = (): React.JSX.Element => {
  const [args, updateArgs] = useArgs<SegmentedControlProps>()

  const handleChange: SegmentedControlProps['onChange'] = useCallback(
    (value) => {
      args.onChange(value)
      updateArgs({ value })
    },
    [args, updateArgs],
  )

  return <SegmentedControl<string> disabled={args.disabled ?? false} name={args.name} onChange={handleChange} options={args.options} value={args.value} />
}

export const Interactive: Story = {
  render: InteractiveSegmentedControl,
}
