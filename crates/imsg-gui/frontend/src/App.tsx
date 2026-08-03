import { useCallback, useState } from 'react'
import AppReady from '@/AppReady.tsx'
import GateConnected from '@/gate/connected/GateConnected.tsx'

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
