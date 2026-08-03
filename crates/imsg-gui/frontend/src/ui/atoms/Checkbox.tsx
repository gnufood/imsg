import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface CheckboxProps {
  checked: boolean
  disabled?: boolean
  label: string
  onChange: (checked: boolean) => void
}

const Checkbox = ({ checked, disabled = false, label, onChange }: CheckboxProps): React.JSX.Element => {
  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onChange(event.target.checked)
    },
    [onChange],
  )

  return (
    <label className={`flex items-center gap-2 text-sm text-ink ${interactiveStyles.disabledContainer}`}>
      <input
        checked={checked}
        className={`size-4 rounded border-line accent-accent ${interactiveStyles.focus}`}
        disabled={disabled}
        onChange={handleChange}
        type="checkbox"
      />
      {label}
    </label>
  )
}

export default Checkbox
