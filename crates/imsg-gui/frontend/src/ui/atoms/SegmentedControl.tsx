import type { LucideIcon } from 'lucide-react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useMemo } from 'react'

interface SegmentedControlOption<Value extends string> {
  label: string
  value: Value
  icon?: LucideIcon
}

interface SegmentedControlProps<Value extends string> {
  disabled?: boolean
  name: string
  onChange: (value: Value) => void
  options: SegmentedControlOption<Value>[]
  value: Value
}

interface Segment<Value extends string> {
  handleChange: () => void
  icon: LucideIcon | undefined
  label: string
  value: Value
}

const buildSegments = <Value extends string>(
  options: SegmentedControlOption<Value>[],
  onChange: (value: Value) => void,
): Segment<Value>[] =>
  options.map((option) => ({
    handleChange: () => {
      onChange(option.value)
    },
    icon: option.icon,
    label: option.label,
    value: option.value,
  }))

const SegmentedControl = <Value extends string>({
  disabled = false,
  name,
  onChange,
  options,
  value,
}: SegmentedControlProps<Value>): React.JSX.Element => {
  const segments = useMemo(() => buildSegments(options, onChange), [onChange, options])

  return (
    <div className="inline-flex gap-0.5">
      {segments.map((segment) => {
        const checked = segment.value === value
        let toneClass = `text-muted ${interactiveStyles.hover}`
        if (checked) {
          toneClass = 'bg-accent text-surface'
        }
        const Icon = segment.icon

        return (
          <label
            key={segment.value}
            className={`inline-flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1 text-sm ${toneClass} ${interactiveStyles.hasFocus} ${interactiveStyles.disabledContainer}`}
          >
            <input
              checked={checked}
              className="sr-only"
              disabled={disabled}
              name={name}
              onChange={segment.handleChange}
              type="radio"
              value={segment.value}
            />
            {Icon !== undefined && <Icon className="size-4" aria-hidden="true" />}
            {segment.label}
          </label>
        )
      })}
    </div>
  )
}

export default SegmentedControl
