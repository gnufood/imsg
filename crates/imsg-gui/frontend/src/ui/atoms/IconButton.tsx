import type { LucideIcon } from 'lucide-react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface IconButtonProps {
  onClick: () => void
  icon: LucideIcon
  // No visible text label, so this is the only accessible name.
  label: string
  disabled?: boolean
}

// Icon-only control (pane collapse/expand toggles, etc.) — distinct from Button's bordered
// Text-pill shape, so it's its own atom rather than an optional-children variant of Button.
const IconButton = ({ onClick, icon: Icon, label, disabled = false }: IconButtonProps): React.JSX.Element => (
  <button
    type="button"
    aria-label={label}
    disabled={disabled}
    className={`grid size-7 shrink-0 place-items-center rounded-md text-muted hover:text-ink ${interactiveStyles.hover} ${interactiveStyles.focus} ${interactiveStyles.disabled} ${interactiveStyles.disabledHoverReset}`}
    onClick={onClick}
  >
    <Icon className="size-4" aria-hidden="true" />
  </button>
)

export default IconButton
