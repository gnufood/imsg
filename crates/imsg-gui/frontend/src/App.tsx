import { useCallback, useMemo, useState } from 'react'
import AppNav from '@/ui/molecules/AppNav.tsx'
import AppShellTemplate from '@/ui/templates/AppShellTemplate.tsx'
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

  // Wrapped in `useMemo` — same reason as `GateConnected`'s `deviceSetupSlot`: avoids
  // Remounting `AppNav` (and the screen beneath it doesn't depend on this identity anyway) on
  // Every unrelated re-render.
  const nav = useMemo(() => <AppNav activeScreen={screen} onSelectScreen={setScreen} />, [screen])

  if (!ready) {
    return <GateConnected onReady={handleReady} />
  }

  return (
    <AppShellTemplate nav={nav}>
      {screen === 'messages' && <MessagesConnected />}
      {screen === 'settings' && <SettingsConnected />}
    </AppShellTemplate>
  )
}

export default App
