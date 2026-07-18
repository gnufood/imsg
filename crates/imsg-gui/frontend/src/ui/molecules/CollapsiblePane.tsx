import { AnimatePresence, motion } from 'motion/react'
import { useMemo } from 'react'

interface CollapsiblePaneProps {
  children: React.ReactNode
  collapsed: boolean
  collapsedWidth: number
  // Applied to the swapped-in content node itself (header/list, or the conversation view) —
  // Not the shell, which only carries the border/scroll/flex-sizing bits that never change.
  contentClassName: string
  expandedWidth: number | 'auto'
  rail: React.ReactNode
  // Static across `collapsed` — no width utility here; width is driven by the `animate` prop
  // Below so it's a real `width` transition, not a `layout`-projection scale.
  shellClassName: string
}

const WIDTH_TRANSITION = { duration: 0.2 }
// Shorter than `WIDTH_TRANSITION` on purpose — the swapped content should be fully faded out
// Before the shell finishes resizing, so there's nothing visible left to look distorted while
// The shell is still animating.
const FADE_TRANSITION = { duration: 0.12 }
const FADE_HIDDEN = { opacity: 0 }
const FADE_VISIBLE = { opacity: 1 }

// Two nested motion nodes, deliberately not one: the shell only ever animates `width` (a real
// Property, so content reflows instead of being scaled — scaling a wrapper with real text/icons
// Inside visibly squishes them) and clips overflow, while the swapped content independently
// Fades on its own faster timeline. `mode="wait"` serializes the swap so the rail and the full
// Content are never both partially visible fighting for the same space.
const CollapsiblePane = ({
  children,
  collapsed,
  collapsedWidth,
  contentClassName,
  expandedWidth,
  rail,
  shellClassName,
}: CollapsiblePaneProps): React.JSX.Element => {
  let targetWidth: number | 'auto' = expandedWidth
  if (collapsed) {
    targetWidth = collapsedWidth
  }
  const widthAnimation = useMemo(() => ({ width: targetWidth }), [targetWidth])

  let content = (
    <motion.div key="expanded" animate={FADE_VISIBLE} className={contentClassName} exit={FADE_HIDDEN} initial={FADE_HIDDEN} transition={FADE_TRANSITION}>
      {children}
    </motion.div>
  )
  if (collapsed) {
    content = (
      <motion.div key="collapsed" animate={FADE_VISIBLE} exit={FADE_HIDDEN} initial={FADE_HIDDEN} transition={FADE_TRANSITION}>
        {rail}
      </motion.div>
    )
  }

  return (
    <motion.div animate={widthAnimation} className={`overflow-x-hidden ${shellClassName}`} initial={false} transition={WIDTH_TRANSITION}>
      <AnimatePresence initial={false} mode="wait">
        {content}
      </AnimatePresence>
    </motion.div>
  )
}

export default CollapsiblePane
