import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface ButtonProps {
  onClick: () => void
  icon?: LucideIcon
  disabled?: boolean
  children: ReactNode
  // Borderless variant — for controls that shouldn't read as boxed actions (e.g. paired
  // With another control right next to them, like `DaemonControls`' install/uninstall row).
  ghost?: boolean
}

// Hardcoded `type="button"` — this never triggers a form submit.
const Button = ({ onClick, icon: Icon, disabled = false, children, ghost = false }: ButtonProps): React.JSX.Element => {
  let variantClass = 'rounded-md border border-line px-3.5 py-1.5'
  if (ghost) {
    variantClass = 'rounded-md px-2 py-1'
  }

  return (
    <button
      type="button"
      disabled={disabled}
      className={`inline-flex items-center gap-2 text-sm text-ink ${variantClass} ${interactiveStyles.hover} ${interactiveStyles.focus} ${interactiveStyles.disabled} ${interactiveStyles.disabledHoverReset}`}
      onClick={onClick}
    >
      {Icon !== undefined && <Icon className="size-4" />}
      {children}
    </button>
  )
}

export default Button
