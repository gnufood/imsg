import { PanelLeftClose, PanelLeftOpen, RefreshCw } from 'lucide-react'
import CollapsedRail from '@/ui/molecules/CollapsedRail.tsx'
import CollapsiblePane from '@/ui/molecules/CollapsiblePane.tsx'
import PaneHeader from '@/ui/molecules/PaneHeader.tsx'
import type { ThreadDto } from '@/bindings.ts'
import ThreadList from '@/messages/organisms/ThreadList.tsx'
import { useMemo } from 'react'

interface ThreadListPaneProps {
  collapsed: boolean
  onRefreshContacts: () => void
  onSelect: (address: string) => void
  onToggle: () => void
  refreshingContacts: boolean
  threads: ThreadDto[]
}

const RAIL_WIDTH = 48
const EXPANDED_WIDTH = 288

// `border-r` stays regardless of collapsed state — it's the divider between panes, not tied to
// Either side's own state.
const ThreadListPane = ({ collapsed, onRefreshContacts, onSelect, onToggle, refreshingContacts, threads }: ThreadListPaneProps): React.JSX.Element => {
  // Avoids remounting the rail's `AnimatePresence` child on every unrelated re-render of this
  // Component — same reasoning as `AppTemplate`'s memoized `nav`.
  const rail = useMemo(() => <CollapsedRail expandIcon={PanelLeftOpen} expandLabel="Expand conversations panel" onExpand={onToggle} />, [onToggle])

  return (
    <CollapsiblePane
      collapsed={collapsed}
      collapsedWidth={RAIL_WIDTH}
      contentClassName="flex flex-col gap-4 p-4"
      expandedWidth={EXPANDED_WIDTH}
      rail={rail}
      shellClassName="flex shrink-0 flex-col overflow-y-auto border-r border-line"
    >
      <PaneHeader
        collapseIcon={PanelLeftClose}
        collapseLabel="Collapse conversations panel"
        onCollapse={onToggle}
        onSecondaryAction={onRefreshContacts}
        secondaryIcon={RefreshCw}
        secondaryLabel="Refresh contacts"
        secondaryPending={refreshingContacts}
        title="Conversations"
      />
      <ThreadList threads={threads} onSelect={onSelect} />
    </CollapsiblePane>
  )
}

export default ThreadListPane
