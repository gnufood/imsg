import { Inbox } from 'lucide-react'
import Text from '@/ui/atoms/Text.tsx'

interface EmptyStateProps {
  message: string
  icon?: typeof Inbox
}

const EmptyState = ({ message, icon: Icon = Inbox }: EmptyStateProps): React.JSX.Element => (
  <div className="flex flex-col items-center gap-3">
    <Icon className="size-7 text-muted" aria-hidden="true" />
    <Text tone="muted">{message}</Text>
  </div>
)

export default EmptyState
