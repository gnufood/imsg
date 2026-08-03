import { AnimatePresence, motion } from 'motion/react'
import AppNav from '@/ui/molecules/AppNav.tsx'
import AppShellTemplate from '@/ui/templates/AppShellTemplate.tsx'
import { useMemo } from 'react'

// Not shared via export — `import/no-named-export` disallows it here, and it's a trivial
// Two-string union `App.tsx` can just redeclare structurally rather than needing a `.types.ts`.
type Screen = 'messages' | 'settings'

interface AppTemplateProps {
  // Pre-built by the caller (same `deviceSetupSlot` precedent as `GateTemplate`) — this template
  // Never knows these are real-IPC `MessagesConnected`/`SettingsConnected` wrappers.
  messagesSlot: React.JSX.Element
  onSelectScreen: (screen: Screen) => void
  screen: Screen
  settingsSlot: React.JSX.Element
}

const FADE_TRANSITION = { duration: 0.15 }
const FADE_HIDDEN = { opacity: 0 }
const FADE_VISIBLE = { opacity: 1 }

const AppTemplate = ({ messagesSlot, onSelectScreen, screen, settingsSlot }: AppTemplateProps): React.JSX.Element => {
  // Same reason as the `App.tsx` version this was lifted from: avoids remounting `AppNav` on
  // Every unrelated re-render of this template.
  const nav = useMemo(() => <AppNav activeScreen={screen} onSelectScreen={onSelectScreen} />, [screen, onSelectScreen])

  return (
    <AppShellTemplate nav={nav}>
      {/* `mode="wait"` — the two screens' layouts differ enough that crossfading them
      simultaneously would overlap oddly; finishing the exit before the enter starts avoids that. */}
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
