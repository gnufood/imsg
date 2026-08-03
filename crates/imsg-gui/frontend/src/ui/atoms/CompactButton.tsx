import type { ReactNode } from 'react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface CompactButtonProps {
  onClick: () => void
  disabled?: boolean
  children: ReactNode
  ghost?: boolean
}

const CompactButton = ({ onClick, disabled = false, children, ghost = false }: CompactButtonProps): React.JSX.Element => {
  let variantClass = 'rounded border border-line px-2 py-0.5'
  if (ghost) {
    variantClass = 'rounded px-1 py-0.5'
  }

  return (
    <button
      type="button"
      disabled={disabled}
      className={`text-xs text-ink ${variantClass} ${interactiveStyles.hover} ${interactiveStyles.focus} ${interactiveStyles.disabled} ${interactiveStyles.disabledHoverReset}`}
      onClick={onClick}
    >
      {children}
    </button>
  )
}

export default CompactButton
