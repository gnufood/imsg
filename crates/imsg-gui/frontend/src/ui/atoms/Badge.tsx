import type { LucideIcon } from 'lucide-react'

interface BadgeProps {
  count: number
  icon?: LucideIcon
  // 'corner' overlays the badge on a positioned ancestor's top-right edge (e.g. ThreadListItem's
  // Unread marker on its row) instead of taking up space in normal flow.
  position?: 'corner' | 'inline'
}

// The icon variant overhangs the corner slightly (negative offset pushes it up/right past the
// Row's edge) — it has no padding of its own (see `variantClass` below) to push it there instead.
// The pill variant keeps its own, larger overhang.
const CORNER_CLASSES: Record<'icon' | 'pill', string> = {
  icon: 'absolute -top-0.5 -right-0.5',
  pill: 'absolute -top-1 -right-1',
}

// Generic count marker (unread badges today; not tied to any one feature). Icon is caller-picked,
// Same optional-icon shape as Button. The pill (fill/rounding/padding) only applies without an
// Icon — with one, the icon's own stroke carries the accent color instead of a separately sized
// Pill behind it.
const Badge = ({ count, icon: Icon, position = 'inline' }: BadgeProps): React.JSX.Element => {
  let variantClass = 'size-5'
  let cornerClass = CORNER_CLASSES.icon
  if (Icon === undefined) {
    variantClass = 'rounded-full bg-accent px-1.5 py-0.5 text-xs font-medium text-surface'
    cornerClass = CORNER_CLASSES.pill
  }

  let positionClass = 'relative'
  if (position === 'corner') {
    positionClass = cornerClass
  }

  return (
    <span className={`inline-flex shrink-0 items-center justify-center ${variantClass} ${positionClass}`}>
      {Icon === undefined && <span>{count}</span>}
      {Icon !== undefined && (
        <>
          <Icon className="size-5 text-accent" aria-hidden="true" />
          <span className="absolute text-[10px] font-medium text-accent">{count}</span>
        </>
      )}
    </span>
  )
}

export default Badge
