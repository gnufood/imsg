import { AnimatePresence, motion } from 'motion/react'
import { useCallback, useState } from 'react'
import AppReady from '@/AppReady.tsx'
import GateConnected from '@/gate/connected/GateConnected.tsx'
import motionTokens from '@/ui/atoms/motion.ts'

const APP_INITIAL = { opacity: 0 }
const APP_ANIMATE = { opacity: 1 }
const GATE_EXIT = { opacity: 0 }

const App = (): React.JSX.Element => {
  const [ready, setReady] = useState(false)

  const handleReady = useCallback(() => {
    setReady(true)
  }, [])

  if (ready) {
    return (
      <AnimatePresence mode="wait">
        <motion.div key="app" animate={APP_ANIMATE} initial={APP_INITIAL} transition={motionTokens.fadeTransition}>
          <AppReady />
        </motion.div>
      </AnimatePresence>
    )
  }

  return (
    <AnimatePresence mode="wait">
      <motion.div key="gate" exit={GATE_EXIT} transition={motionTokens.fadeTransition}>
        <GateConnected onReady={handleReady} />
      </motion.div>
    </AnimatePresence>
  )
}

export default App
