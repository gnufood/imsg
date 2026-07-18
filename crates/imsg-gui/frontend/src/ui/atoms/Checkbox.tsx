import { Square, SquareCheck } from 'lucide-react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface CheckboxProps {
  checked: boolean
  disabled?: boolean
  label: string
  onChange: (checked: boolean) => void
}

// The native `<input>` stays for keyboard/click/screen-reader semantics but is visually
// Hidden (`sr-only`) — the checked/unchecked glyph is a lucide icon instead of the browser's
// Own checkbox rendering, so it follows our tokens like every other atom. Label-wraps-input
// Click delegation and tab order both still work on a `sr-only` (not `hidden`) input.
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
