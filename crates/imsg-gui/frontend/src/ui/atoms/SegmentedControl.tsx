import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useMemo } from 'react'

interface SegmentedControlOption<Value extends string> {
  label: string
  value: Value
}

interface SegmentedControlProps<Value extends string> {
  disabled?: boolean
  // Groups the native radios — required since a screen can host more than one instance.
  name: string
  onChange: (value: Value) => void
  options: SegmentedControlOption<Value>[]
  value: Value
}

interface Segment<Value extends string> {
  handleChange: () => void
  label: string
  value: Value
}

// Stable per-option handlers, built once per `options`/`onChange` identity rather than as a
// Fresh closure per render inside the JSX below (see `jsx-no-new-function-as-prop`).
const buildSegments = <Value extends string>(
  options: SegmentedControlOption<Value>[],
  onChange: (value: Value) => void,
): Segment<Value>[] =>
  options.map((option) => ({
    handleChange: () => {
      onChange(option.value)
    },
    label: option.label,
    value: option.value,
  }))

// Native `<input type="radio">` per option stays for keyboard (arrow-key group navigation is
// Free from the browser) and screen-reader semantics, visually hidden — same `sr-only` +
// Custom-visual pattern as `Checkbox`, styled as a tab-like segment instead of an icon.
const SegmentedControl = <Value extends string>({
  disabled = false,
  name,
  onChange,
  options,
  value,
}: SegmentedControlProps<Value>): React.JSX.Element => {
  const segments = useMemo(() => buildSegments(options, onChange), [onChange, options])

  return (
    <div className="inline-flex gap-0.5 rounded-md border border-line p-0.5">
      {segments.map((segment) => {
        const checked = segment.value === value
        let toneClass = `text-muted ${interactiveStyles.hover}`
        if (checked) {
          toneClass = 'bg-accent text-surface'
        }

        return (
          <label
            key={segment.value}
            className={`cursor-pointer rounded-[5px] px-2.5 py-1 text-sm ${toneClass} ${interactiveStyles.hasFocus} ${interactiveStyles.disabledContainer}`}
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
            {segment.label}
          </label>
        )
      })}
    </div>
  )
}

export default SegmentedControl
