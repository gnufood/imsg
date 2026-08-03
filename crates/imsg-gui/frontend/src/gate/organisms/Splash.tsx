import { useCallback, useEffect, useMemo, useState } from 'react'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import { motion } from 'motion/react'

const MIN_FLOOR_MS = 3800
const CAP_MS = 1750
const FADE_SECONDS = 0.3
const FADE_TRANSITION = { duration: FADE_SECONDS }

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
        transition={FADE_TRANSITION}
        onAnimationComplete={handleFadeComplete}
      />
    </CenteredScreen>
  )
}

export default Splash
