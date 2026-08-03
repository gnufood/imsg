import type { LucideIcon } from 'lucide-react'

interface BadgeProps {
  count: number
  icon?: LucideIcon
  position?: 'corner' | 'inline'
}

const CORNER_CLASSES: Record<'icon' | 'pill', string> = {
  icon: 'absolute -top-0.5 -right-0.5',
  pill: 'absolute -top-1 -right-1',
}

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
