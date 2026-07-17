import { useCallback } from 'react'

interface TextInputProps {
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  onKeyDown?: (event: React.KeyboardEvent<HTMLInputElement>) => void
  placeholder?: string
}

const TextInput = ({ value, onChange, disabled = false, onKeyDown, placeholder }: TextInputProps): React.JSX.Element => {
  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onChange(event.target.value)
    },
    [onChange],
  )

  return (
    <input
      type="text"
      value={value}
      onChange={handleChange}
      onKeyDown={onKeyDown}
      placeholder={placeholder}
      disabled={disabled}
      className="w-full rounded-md border border-line bg-transparent px-3.5 py-2 text-sm text-ink placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent disabled:cursor-not-allowed disabled:opacity-50"
    />
  )
}

export default TextInput
