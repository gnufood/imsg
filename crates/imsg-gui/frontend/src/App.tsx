import { useCallback, useMemo, useState } from 'react'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import GateConnected from '@/gate/connected/GateConnected.tsx'
import MessagesConnected from '@/messages/connected/MessagesConnected.tsx'
import SettingsConnected from '@/settings/connected/SettingsConnected.tsx'

type Screen = 'messages' | 'settings'

// Gate mirrors the backend's startup sequence (device config → daemon → sync, owned by
// `gate::run` in Rust) and hands off once it reports Ready. `screen` is the persistent app
// Shell's nav state — only meaningful once `ready`; Gate/Splash/DeviceSetup stay full-bleed.
const App = (): React.JSX.Element => {
  const [ready, setReady] = useState(false)
  const [screen, setScreen] = useState<Screen>('messages')

  const handleReady = useCallback(() => {
    setReady(true)
  }, [])

  // Both slots mount once `ready` and never depend on anything that changes afterward — same
  // `deviceSetupSlot` precedent as `GateConnected`, just with empty deps instead of one.
  const messagesSlot = useMemo(() => <MessagesConnected />, [])
  const settingsSlot = useMemo(() => <SettingsConnected />, [])

  if (!ready) {
    return <GateConnected onReady={handleReady} />
  }

  return <AppTemplate messagesSlot={messagesSlot} onSelectScreen={setScreen} screen={screen} settingsSlot={settingsSlot} />
}

export default App
