import { useCallback, useEffect, useMemo, useState } from 'react'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import { motion } from 'motion/react'

// GUI_FRONTEND.md's timing spec: start the async check and the animation together, wait for
// Both a minimum floor and the check (capped, so a slow check can't hang the splash forever),
// Fade out on whichever finishes later.
//
// Floor is measured off the actual asset, not the doc's original 400-500ms placeholder:
// Public/splash.webp's logo reveal draws the "R" mark first, settling at frame 86/212
// (~3.6s into its ~8.64s runtime) before the chain/hand elements layer on top. The floor
// Must clear that regardless of `ready`/`capped` — see `fading` below — so the reveal is
// Never cut off mid-stroke. `CAP_MS` staying under this floor is fine: it only bounds how
// Long we wait on `ready` specifically, and the floor already dominates total wait time.
const MIN_FLOOR_MS = 3800
const CAP_MS = 1750
// Motion (not a Tailwind transition class) drives the fade, so this is its only source of
// Truth — no CSS-side duplicate to keep in sync, unlike the old CSS-transition version.
const FADE_SECONDS = 0.3
// Static — hoisted out of the component so it's a stable reference, not a fresh object every
// Render (Motion re-runs the transition whenever this prop's identity changes).
const FADE_TRANSITION = { duration: FADE_SECONDS }

interface SplashProps {
  // Caller-owned async check (e.g. `Gate`'s `configIsDeviceConfigured`); Splash only times
  // Against it, never awaits or interprets it itself — no IPC awareness here.
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

  // Fires when Motion's own fade animation finishes — no separate `setTimeout` needed to
  // Guess when it's done, unlike the old CSS-transition version.
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
