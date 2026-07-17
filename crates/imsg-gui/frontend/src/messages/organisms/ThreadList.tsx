import EmptyState from '@/ui/molecules/EmptyState.tsx'
import type { ThreadDto } from '@/bindings.ts'
import ThreadListItem from '@/ui/molecules/ThreadListItem.tsx'

interface ThreadListProps {
  threads: ThreadDto[]
  onSelect: (address: string) => void
}

const ThreadList = ({ threads, onSelect }: ThreadListProps): React.JSX.Element => {
  if (threads.length === 0) {
    return <EmptyState message="No conversations yet." />
  }

  return (
    <ul className="flex w-full flex-col gap-2">
      {threads.map((thread) => (
        <ThreadListItem key={thread.address} thread={thread} onSelect={onSelect} />
      ))}
    </ul>
  )
}

export default ThreadList
