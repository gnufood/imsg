import type { LucideIcon } from 'lucide-react'

interface BadgeProps {
  count: number
  icon?: LucideIcon
  // 'corner' overlays the badge on a positioned ancestor's top-right edge (e.g. ThreadListItem's
  // Unread marker on its row) instead of taking up space in normal flow.
  position?: 'corner' | 'inline'
}

const POSITION_CLASSES: Record<NonNullable<BadgeProps['position']>, string> = {
  corner: 'absolute -top-1 -right-1',
  inline: 'relative',
}

// Generic count pill (unread badges today; not tied to any one feature). Icon is caller-picked,
// Same optional-icon shape as Button.
const Badge = ({ count, icon: Icon, position = 'inline' }: BadgeProps): React.JSX.Element => (
  <span className={`inline-flex shrink-0 items-center justify-center rounded-full bg-accent px-1.5 py-0.5 text-xs font-medium text-surface ${POSITION_CLASSES[position]}`}>
    {Icon !== undefined && <Icon className="size-5" aria-hidden="true" />}
    {Icon === undefined && <span>{count}</span>}
    {Icon !== undefined && <span className="absolute text-[10px]">{count}</span>}
  </span>
)

export default Badge
