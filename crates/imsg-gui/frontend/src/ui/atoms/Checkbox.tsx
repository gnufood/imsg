import { Square, SquareCheck } from 'lucide-react'
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

  let Icon = Square
  let toneClass = 'text-muted'
  if (checked) {
    Icon = SquareCheck
    toneClass = 'text-accent'
  }

  return (
    <label className={`flex items-center gap-2 text-sm text-ink ${interactiveStyles.disabledContainer}`}>
      <input checked={checked} className="peer sr-only" disabled={disabled} onChange={handleChange} type="checkbox" />
      <Icon aria-hidden="true" className={`size-4 shrink-0 ${toneClass} ${interactiveStyles.peerFocus}`} />
      {label}
    </label>
  )
}

export default Checkbox
