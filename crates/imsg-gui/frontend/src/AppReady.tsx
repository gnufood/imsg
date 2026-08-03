import { useMemo, useState } from 'react'
import AppTemplate from '@/ui/templates/AppTemplate.tsx'
import MessagesConnected from '@/messages/connected/MessagesConnected.tsx'
import SettingsConnected from '@/settings/connected/SettingsConnected.tsx'
import useThemePreference from '@/theme/application/use-theme-preference.ts'

type Screen = 'messages' | 'settings'

const AppReady = (): React.JSX.Element => {
  const [screen, setScreen] = useState<Screen>('messages')
  const { preference, setPreference } = useThemePreference()

  const messagesSlot = useMemo(() => <MessagesConnected />, [])
  const settingsSlot = useMemo(
    () => <SettingsConnected onPreferenceChange={setPreference} preference={preference} />,
    [preference, setPreference],
  )

  return <AppTemplate messagesSlot={messagesSlot} onSelectScreen={setScreen} screen={screen} settingsSlot={settingsSlot} />
}

export default AppReady
