import { useCallback, useState } from 'react'
import GateConnected from '@/gate/connected/GateConnected.tsx'

// Gate mirrors the backend's startup sequence (device config → daemon → sync, owned by
// `gate::run` in Rust) and hands off once it reports Ready. The post-gate main app isn't
// Built yet — this renders a bare placeholder in its place.
const App = (): React.JSX.Element => {
  const [ready, setReady] = useState(false)

  const handleReady = useCallback(() => {
    setReady(true)
  }, [])

  if (!ready) {
    return <GateConnected onReady={handleReady} />
  }

  return <p className="p-6 text-sm text-muted">imsg is ready — main app not built yet.</p>
}

export default App
