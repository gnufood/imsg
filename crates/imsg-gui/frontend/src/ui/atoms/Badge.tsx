import type { LucideIcon } from 'lucide-react'

interface BadgeProps {
  count: number
  icon?: LucideIcon
}

// Generic count pill (unread badges today; not tied to any one feature). Icon is caller-picked,
// Same optional-icon shape as Button.
const Badge = ({ count, icon: Icon }: BadgeProps): React.JSX.Element => (
  <span className="relative inline-flex shrink-0 items-center justify-center rounded-full bg-accent px-1.5 py-0.5 text-xs font-medium text-surface">
    {Icon !== undefined && <Icon className="size-5" aria-hidden="true" />}
    {Icon === undefined && <span>{count}</span>}
    {Icon !== undefined && <span className="absolute text-[10px]">{count}</span>}
  </span>
)

export default Badge
