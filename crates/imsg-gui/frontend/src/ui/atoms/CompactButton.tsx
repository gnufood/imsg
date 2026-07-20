import type { ReactNode } from 'react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface CompactButtonProps {
  onClick: () => void
  disabled?: boolean
  children: ReactNode
  // Borderless variant, same meaning as `Button`'s own `ghost` — for the de-emphasized action in
  // A row of otherwise-bordered compact buttons (e.g. ChannelOverridesForm's "Detect from device").
  ghost?: boolean
}

// Thinner sibling to `Button` — for dense inline action rows (ChannelOverridesForm's
// Detect/Cancel/Apply) where `Button`'s own padding (already tightened once for height, see
// 6a0722f) still reads too tall next to each other. Kept as its own atom rather than a `Button`
// Size variant so `Button`'s other callers (ConfirmDialog, ServiceStatusCard, ErrorState) are
// Unaffected.
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
