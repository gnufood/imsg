import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'

interface ButtonProps {
  onClick: () => void
  icon?: LucideIcon
  disabled?: boolean
  children: ReactNode
}

// Hardcoded `type="button"` — this never triggers a form submit.
const Button = ({ onClick, icon: Icon, disabled = false, children }: ButtonProps): React.JSX.Element => (
  <button
    type="button"
    disabled={disabled}
    className="inline-flex items-center gap-2 rounded-md border border-line px-3.5 py-2 text-sm text-ink hover:bg-ink/5 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent"
    onClick={onClick}
  >
    {Icon !== undefined && <Icon className="size-4" />}
    {children}
  </button>
)

export default Button
