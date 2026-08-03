import IconButton from '@/ui/atoms/IconButton.tsx'
import type { LucideIcon } from 'lucide-react'

interface CollapsedRailProps {
  onExpand: () => void
  expandIcon: LucideIcon
  expandLabel: string
}

// Narrow strip a pane shrinks to while collapsed — just its expand toggle, vertically centered.
const CollapsedRail = ({ onExpand, expandIcon, expandLabel }: CollapsedRailProps): React.JSX.Element => (
  <div className="flex h-full flex-col items-center pt-4">
    <IconButton icon={expandIcon} label={expandLabel} onClick={onExpand} />
  </div>
)

export default CollapsedRail
