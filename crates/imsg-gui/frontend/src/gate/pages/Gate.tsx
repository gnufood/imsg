import { useCallback, useEffect, useState } from 'react'
import type { GateStatus } from '@/bindings.ts'
import GateTemplate from '@/gate/templates/GateTemplate.tsx'

interface GateProps {
  deviceSetupSlot: React.JSX.Element
  onProceed: () => void
  onReady: () => void
  onResumePolling: () => void
  pollFailed: boolean
  status: GateStatus | undefined
}

const Gate = ({ deviceSetupSlot, onProceed, onReady, onResumePolling, pollFailed, status }: GateProps): React.JSX.Element => {
  const [splashDone, setSplashDone] = useState(false)

  const handleSplashDone = useCallback(() => {
    setSplashDone(true)
  }, [])

  useEffect(() => {
    if (splashDone && status === 'Ready') {
      onReady()
    }
  }, [splashDone, status, onReady])

  return (
    <GateTemplate
      deviceSetupSlot={deviceSetupSlot}
      onProceed={onProceed}
      onResumePolling={onResumePolling}
      onSplashDone={handleSplashDone}
      pollFailed={pollFailed}
      splashDone={splashDone}
      status={status}
    />
  )
}

export default Gate
