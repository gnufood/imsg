import IconButton from '@/ui/atoms/IconButton.tsx'
import type { LucideIcon } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'

interface PaneHeaderProps {
  title: string
  onCollapse: () => void
  collapseIcon: LucideIcon
  collapseLabel: string
  // Optional second action alongside collapse (e.g. `ThreadListPane`'s refresh-contacts button).
  // All three must be given together — enforced by `renderSecondaryAction` below, not the type,
  // Since a partial optional trio isn't expressible without a discriminated union that would
  // Complicate every existing caller that doesn't need one.
  onSecondaryAction?: () => void
  secondaryIcon?: LucideIcon
  secondaryLabel?: string
  secondaryPending?: boolean
}

interface SecondaryActionArgs {
  onSecondaryAction: (() => void) | undefined
  secondaryIcon: LucideIcon | undefined
  secondaryLabel: string | undefined
  secondaryPending: boolean | undefined
}

const renderSecondaryAction = ({ onSecondaryAction, secondaryIcon, secondaryLabel, secondaryPending = false }: SecondaryActionArgs): React.JSX.Element | undefined => {
  if (onSecondaryAction === undefined || secondaryIcon === undefined || secondaryLabel === undefined) {
    return undefined
  }
  return <IconButton disabled={secondaryPending} icon={secondaryIcon} label={secondaryLabel} onClick={onSecondaryAction} />
}

const PaneHeader = ({
  title,
  onCollapse,
  collapseIcon,
  collapseLabel,
  onSecondaryAction,
  secondaryIcon,
  secondaryLabel,
  secondaryPending,
}: PaneHeaderProps): React.JSX.Element => (
  <div className="flex items-center justify-between">
    <Text as="span" tone="muted">
      {title}
    </Text>
    <div className="flex items-center gap-1">
      {renderSecondaryAction({ onSecondaryAction, secondaryIcon, secondaryLabel, secondaryPending })}
      <IconButton icon={collapseIcon} label={collapseLabel} onClick={onCollapse} />
    </div>
  </div>
)

export default PaneHeader
