import type { ReactNode } from 'react'

type TextTone = 'ink' | 'muted' | 'surface'
type TextSize = 'sm' | 'xs'

const TONE_CLASS: Record<TextTone, string> = {
  ink: 'text-ink',
  muted: 'text-muted',
  surface: 'text-surface',
}

const SIZE_CLASS: Record<TextSize, string> = {
  sm: 'text-sm',
  xs: 'text-xs',
}

interface TextProps {
  children: ReactNode
  // 'span' for text nested inside a `<button>` (e.g. DeviceListItem) — `<p>` isn't valid
  // Phrasing content there. 'h1'/'h2' for section/screen headings (e.g. SettingsTemplate).
  as?: 'p' | 'span' | 'h1' | 'h2'
  size?: TextSize
  tone?: TextTone
}

const Text = ({ children, as: Tag = 'p', size = 'sm', tone = 'ink' }: TextProps): React.JSX.Element => (
  <Tag className={`${SIZE_CLASS[size]} ${TONE_CLASS[tone]}`}>{children}</Tag>
)

export default Text
