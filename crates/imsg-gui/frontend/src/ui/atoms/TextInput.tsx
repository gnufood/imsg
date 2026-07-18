import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface TextInputProps {
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  // For explicit `<label htmlFor>` pairing when the field isn't nested inside its own label.
  id?: string
  onKeyDown?: (event: React.KeyboardEvent<HTMLInputElement>) => void
  placeholder?: string
}

const TextInput = ({ value, onChange, disabled = false, id, onKeyDown, placeholder }: TextInputProps): React.JSX.Element => {
  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onChange(event.target.value)
    },
    [onChange],
  )

  return (
    <input
      type="text"
      id={id}
      value={value}
      onChange={handleChange}
      onKeyDown={onKeyDown}
      placeholder={placeholder}
      disabled={disabled}
      className={`w-full rounded-md border border-line bg-transparent px-3.5 py-1.5 text-sm text-ink placeholder:text-muted ${interactiveStyles.focus} ${interactiveStyles.disabled}`}
    />
  )
}

export default TextInput
