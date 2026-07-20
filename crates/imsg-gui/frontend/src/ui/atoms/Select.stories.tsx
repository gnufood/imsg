import type { Meta, StoryObj } from '@storybook/react-vite'
import Select from '@/ui/atoms/Select.tsx'
import { fn } from 'storybook/test'
import { useArgs } from 'storybook/preview-api'
import { useCallback } from 'react'

const OPTIONS = Array.from({ length: 30 }, (_unused, index) => ({ label: String(index + 1), value: String(index + 1) }))

const meta = {
  args: {
    disabled: false,
    onChange: fn(),
    options: OPTIONS,
    value: '8',
  },
  component: Select,
  title: 'atoms/Select',
} satisfies Meta<typeof Select>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Disabled: Story = {
  args: { disabled: true },
}

type SelectProps = React.ComponentProps<typeof Select>

// `onChange: fn()` alone only lets Storybook's Actions panel log the change — it never feeds
// Back into `value`, so the control would look dead. `useArgs` closes that loop, same pattern as
// `SegmentedControl.stories.tsx`'s `Interactive` story.
const InteractiveSelect = (): React.JSX.Element => {
  const [args, updateArgs] = useArgs<SelectProps>()

  const handleChange: SelectProps['onChange'] = useCallback(
    (value) => {
      args.onChange(value)
      updateArgs({ value })
    },
    [args, updateArgs],
  )

  return <Select disabled={args.disabled ?? false} onChange={handleChange} options={args.options} value={args.value} />
}

export const Interactive: Story = {
  render: InteractiveSelect,
}
