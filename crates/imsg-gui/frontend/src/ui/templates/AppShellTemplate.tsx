import type { ReactNode } from 'react'

interface AppShellTemplateProps {
  children: ReactNode
  nav: React.JSX.Element
}

const AppShellTemplate = ({ children, nav }: AppShellTemplateProps): React.JSX.Element => (
  <div className="flex h-screen w-full flex-col bg-surface text-ink">
    <header className="flex shrink-0 items-center justify-end border-b border-line px-4 py-2">{nav}</header>
    <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
  </div>
)

export default AppShellTemplate
