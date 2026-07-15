import type { ReactNode } from 'react'

interface CenteredScreenProps {
  children: ReactNode
}

const CenteredScreen = ({ children }: CenteredScreenProps): React.JSX.Element => (
  <div className="grid min-h-screen place-items-center gap-4 bg-surface px-6 text-center text-ink">
    {children}
  </div>
)

export default CenteredScreen
