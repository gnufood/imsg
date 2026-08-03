import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import type { SessionState } from '@/bindings.ts'
import StatusDot from '@/ui/atoms/StatusDot.tsx'
import type StatusPanelArgs from '@/settings/organisms/StatusPanel.types.ts'
import Text from '@/ui/atoms/Text.tsx'

type Status = SessionState | null

interface StatusDescription {
  label: string
  tone: 'active' | 'error' | 'idle' | 'pending'
}

const describeStatus = (status: Status): StatusDescription => {
  if (status === null) {
    return { label: 'No daemon detected', tone: 'idle' }
  }
  switch (status) {
    case 'disconnected': {
      return { label: 'Disconnected', tone: 'idle' }
    }
    case 'connecting': {
      return { label: 'Connecting…', tone: 'pending' }
    }
    case 'active': {
      return { label: 'Connected', tone: 'active' }
    }
    case 'reconnecting': {
      return { label: 'Reconnecting…', tone: 'pending' }
    }
    case 'failed': {
      return { label: 'Failed', tone: 'error' }
    }
  }
}

const renderConnection = (status: Status | undefined, statusPollFailed: boolean, onResumeStatusPolling: () => void): React.JSX.Element => {
  if (statusPollFailed) {
    return <ErrorState message="Couldn't check the daemon's connection." onRetry={onResumeStatusPolling} />
  }
  if (status === undefined) {
    return <LoadingState message="Checking connection…" />
  }
  const { label, tone } = describeStatus(status)
  return (
    <div className="flex items-center gap-2">
      <StatusDot tone={tone} />
      <Text size="xs" tone="muted">
        {label}
      </Text>
    </div>
  )
}

const StatusPanel = ({ address, configFailed, onResumeStatusPolling, onRetryConfig, status, statusPollFailed }: StatusPanelArgs): React.JSX.Element => {
  if (configFailed) {
    return <ErrorState message="Couldn't load device config." onRetry={onRetryConfig} />
  }
  if (address === undefined) {
    return <LoadingState message="Loading device info…" />
  }
  return (
    <div className="flex flex-col gap-2">
      <Text>{address}</Text>
      {renderConnection(status, statusPollFailed, onResumeStatusPolling)}
    </div>
  )
}

export default StatusPanel
