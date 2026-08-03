import { BookUser, MessageSquare, Settings as SettingsIcon } from 'lucide-react'
import IconButton from '@/ui/atoms/IconButton.tsx'
import { useCallback } from 'react'

interface AppNavProps {
  activeScreen: 'contacts' | 'messages' | 'settings'
  onSelectScreen: (screen: 'contacts' | 'messages' | 'settings') => void
}

// Disables whichever nav item is already active, rather than a separate "selected" visual
// Variant — no atom currently supports one, and "you're already here" is exactly what `disabled`
// Already communicates.
const AppNav = ({ activeScreen, onSelectScreen }: AppNavProps): React.JSX.Element => {
  const selectMessages = useCallback(() => {
    onSelectScreen('messages')
  }, [onSelectScreen])

  const selectContacts = useCallback(() => {
    onSelectScreen('contacts')
  }, [onSelectScreen])

  const selectSettings = useCallback(() => {
    onSelectScreen('settings')
  }, [onSelectScreen])

  return (
    <nav className="flex items-center gap-1">
      <IconButton disabled={activeScreen === 'messages'} icon={MessageSquare} label="Messages" onClick={selectMessages} />
      <IconButton disabled={activeScreen === 'contacts'} icon={BookUser} label="Contacts" onClick={selectContacts} />
      <IconButton disabled={activeScreen === 'settings'} icon={SettingsIcon} label="Settings" onClick={selectSettings} />
    </nav>
  )
}

export default AppNav
