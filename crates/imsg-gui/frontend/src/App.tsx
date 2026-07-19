import { useCallback, useState } from 'react'
import AppReady from '@/AppReady.tsx'
import GateConnected from '@/gate/connected/GateConnected.tsx'

// Gate mirrors the backend's startup sequence (device config → daemon → sync, owned by
// `gate::run` in Rust) and hands off once it reports Ready. Everything past that point —
// Including `screen` nav state and theming — lives in `AppReady`, mounted only once `ready`;
// Gate/Splash/DeviceSetup stay full-bleed and untouched by either.
const App = (): React.JSX.Element => {
  const [ready, setReady] = useState(false)

  const handleReady = useCallback(() => {
    setReady(true)
  }, [])

  if (!ready) {
    return <GateConnected onReady={handleReady} />
  }

  return <AppReady />
}

export default App
