import { useCallback, useEffect, useMemo, useState } from 'react'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import { motion } from 'motion/react'
import motionTokens from '@/ui/atoms/motion.ts'

// Public/splash.webp: 173 frames, 7080ms total runtime (`webpmux -info`).
const MIN_FLOOR_MS = 7080
const CAP_MS = 1750

interface SplashProps {
  ready: boolean
  onDone: () => void
}

const Splash = ({ ready, onDone }: SplashProps): React.JSX.Element => {
  const [floorPassed, setFloorPassed] = useState(false)
  const [capped, setCapped] = useState(false)

  useEffect(() => {
    const floorTimer = setTimeout(() => setFloorPassed(true), MIN_FLOOR_MS)
    const capTimer = setTimeout(() => setCapped(true), CAP_MS)
    return () => {
      clearTimeout(floorTimer)
      clearTimeout(capTimer)
    }
  }, [])

  const fading = floorPassed && (ready || capped)

  const handleFadeComplete = useCallback(() => {
    if (fading) {
      onDone()
    }
  }, [fading, onDone])

  let opacity = 1
  if (fading) {
    opacity = 0
  }
  const animate = useMemo(() => ({ opacity }), [opacity])

  return (
    <CenteredScreen>
      <motion.img
        src="/splash.webp"
        alt="imsg"
        className="w-80"
        animate={animate}
        transition={motionTokens.fadeTransition}
        onAnimationComplete={handleFadeComplete}
      />
    </CenteredScreen>
  )
}

export default Splash
