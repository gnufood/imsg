import IconButton from '@/ui/atoms/IconButton.tsx'
import type { LucideIcon } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'

interface PaneHeaderProps {
  title: string
  onCollapse: () => void
  collapseIcon: LucideIcon
  collapseLabel: string
}

const PaneHeader = ({ title, onCollapse, collapseIcon, collapseLabel }: PaneHeaderProps): React.JSX.Element => (
  <div className="flex items-center justify-between">
    <Text as="span" tone="muted">
      {title}
    </Text>
    <IconButton icon={collapseIcon} label={collapseLabel} onClick={onCollapse} />
  </div>
)

export default PaneHeader
