import { AnimatePresence, motion } from 'motion/react'
import AppNav from '@/ui/molecules/AppNav.tsx'
import AppShellTemplate from '@/ui/templates/AppShellTemplate.tsx'
import { useMemo } from 'react'

type Screen = 'messages' | 'settings'

interface AppTemplateProps {
  messagesSlot: React.JSX.Element
  onSelectScreen: (screen: Screen) => void
  screen: Screen
  settingsSlot: React.JSX.Element
}

const FADE_TRANSITION = { duration: 0.15 }
const FADE_HIDDEN = { opacity: 0 }
const FADE_VISIBLE = { opacity: 1 }

const AppTemplate = ({ messagesSlot, onSelectScreen, screen, settingsSlot }: AppTemplateProps): React.JSX.Element => {
  const nav = useMemo(() => <AppNav activeScreen={screen} onSelectScreen={onSelectScreen} />, [screen, onSelectScreen])

  return (
    <AppShellTemplate nav={nav}>
      <AnimatePresence initial={false} mode="wait">
        {screen === 'messages' && (
          <motion.div key="messages" animate={FADE_VISIBLE} className="h-full" exit={FADE_HIDDEN} initial={FADE_HIDDEN} transition={FADE_TRANSITION}>
            {messagesSlot}
          </motion.div>
        )}
        {screen === 'settings' && (
          <motion.div key="settings" animate={FADE_VISIBLE} className="h-full" exit={FADE_HIDDEN} initial={FADE_HIDDEN} transition={FADE_TRANSITION}>
            {settingsSlot}
          </motion.div>
        )}
      </AnimatePresence>
    </AppShellTemplate>
  )
}

export default AppTemplate
