import type { ReactNode } from 'react'

interface AppShellTemplateProps {
  children: ReactNode
  // Pre-built by the caller (same `deviceSetupSlot` precedent as `GateTemplate`) — this template
  // Never knows what the nav actually contains, only where it goes.
  nav: React.JSX.Element
}

// Persistent shell wrapping the app once past the gate (see `App.tsx`) — Gate/Splash/DeviceSetup
// Stay full-bleed, never rendered inside this.
const AppShellTemplate = ({ children, nav }: AppShellTemplateProps): React.JSX.Element => (
  <div className="flex h-screen w-full flex-col bg-surface text-ink">
    <header className="flex shrink-0 items-center justify-end border-b border-line px-4 py-2">{nav}</header>
    <div className="min-h-0 flex-1">{children}</div>
  </div>
)

export default AppShellTemplate
