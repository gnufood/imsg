import type { ReactNode } from 'react'

interface CenteredScreenProps {
  children: ReactNode
  // Gate screens render with nothing above them and fill the window. Screens nested inside
  // `AppShellTemplate` sit below its nav, where a viewport-height child overflows the document
  // By exactly the nav's height — explicit so a nested caller can't silently inherit the wrong one.
  fill?: 'parent' | 'viewport'
}

const FILL_CLASS = {
  parent: 'h-full',
  viewport: 'min-h-screen',
} as const

const CenteredScreen = ({ children, fill = 'viewport' }: CenteredScreenProps): React.JSX.Element => (
  <div className={`grid ${FILL_CLASS[fill]} place-items-center gap-4 bg-surface px-6 text-center text-ink`}>
    {children}
  </div>
)

export default CenteredScreen
