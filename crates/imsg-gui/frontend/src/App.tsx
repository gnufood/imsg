import { useCallback, useState } from 'react'
import GateConnected from '@/gate/connected/GateConnected.tsx'
import MessagesConnected from '@/messages/connected/MessagesConnected.tsx'

// Gate mirrors the backend's startup sequence (device config → daemon → sync, owned by
// `gate::run` in Rust) and hands off once it reports Ready.
const App = (): React.JSX.Element => {
  const [ready, setReady] = useState(false)

  const handleReady = useCallback(() => {
    setReady(true)
  }, [])

  if (!ready) {
    return <GateConnected onReady={handleReady} />
  }

  return <MessagesConnected />
}

export default App
