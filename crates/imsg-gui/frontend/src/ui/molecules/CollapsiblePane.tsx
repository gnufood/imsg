import { AnimatePresence, motion } from 'motion/react'
import { useMemo } from 'react'

interface CollapsiblePaneProps {
  children: React.ReactNode
  collapsed: boolean
  collapsedWidth: number
  contentClassName: string
  expandedWidth: number | 'auto'
  rail: React.ReactNode
  shellClassName: string
}

const WIDTH_TRANSITION = { duration: 0.2 }
const FADE_TRANSITION = { duration: 0.12 }
const FADE_HIDDEN = { opacity: 0 }
const FADE_VISIBLE = { opacity: 1 }

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
