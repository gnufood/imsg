import type { LucideIcon } from 'lucide-react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface IconButtonProps {
  onClick: () => void
  icon: LucideIcon
  label: string
  disabled?: boolean
}

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
