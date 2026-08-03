import { useMemo, useState } from 'react'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import ContactsConnected from '@/contacts/connected/ContactsConnected.tsx'
import MessagesConnected from '@/messages/connected/MessagesConnected.tsx'
import SettingsConnected from '@/settings/connected/SettingsConnected.tsx'
import useThemePreference from '@/theme/application/use-theme-preference.ts'

type Screen = 'contacts' | 'messages' | 'settings'

// Split out of `App.tsx` so `useThemePreference` (and anything else post-gate) only ever mounts
// Once `App.tsx`'s gate reports `ready` — mounting, not a runtime branch inside one component, is
// What keeps its `data-theme` effect from ever running during Gate/Splash/DeviceSetup.
const AppReady = (): React.JSX.Element => {
  const [screen, setScreen] = useState<Screen>('messages')
  const { preference, setPreference } = useThemePreference()

  const messagesSlot = useMemo(() => <MessagesConnected />, [])
  const contactsSlot = useMemo(() => <ContactsConnected />, [])
  const settingsSlot = useMemo(
    () => <SettingsConnected onPreferenceChange={setPreference} preference={preference} />,
    [preference, setPreference],
  )

  return (
    <AppTemplate contactsSlot={contactsSlot} messagesSlot={messagesSlot} onSelectScreen={setScreen} screen={screen} settingsSlot={settingsSlot} />
  )
}

export default AppReady
