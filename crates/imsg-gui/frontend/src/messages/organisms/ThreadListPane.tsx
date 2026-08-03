import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import CollapsedRail from '@/ui/molecules/CollapsedRail.tsx'
import PaneHeader from '@/ui/molecules/PaneHeader.tsx'
import type { ThreadDto } from '@/bindings.ts'
import ThreadList from '@/messages/organisms/ThreadList.tsx'

interface ThreadListPaneProps {
  collapsed: boolean
  onSelect: (address: string) => void
  onToggle: () => void
  threads: ThreadDto[]
}

// `border-r` stays regardless of collapsed state — it's the divider between panes, not tied to
// Either side's own state.
const ThreadListPane = ({ collapsed, onSelect, onToggle, threads }: ThreadListPaneProps): React.JSX.Element => {
  if (collapsed) {
    return (
      <div className="flex w-12 shrink-0 flex-col border-r border-line">
        <CollapsedRail expandIcon={PanelLeftOpen} expandLabel="Expand conversations panel" onExpand={onToggle} />
      </div>
    )
  }
  return (
    <div className="flex w-72 shrink-0 flex-col gap-4 overflow-y-auto border-r border-line p-4">
      <PaneHeader collapseIcon={PanelLeftClose} collapseLabel="Collapse conversations panel" onCollapse={onToggle} title="Conversations" />
      <ThreadList threads={threads} onSelect={onSelect} />
    </div>
  )
}

export default ThreadListPane
