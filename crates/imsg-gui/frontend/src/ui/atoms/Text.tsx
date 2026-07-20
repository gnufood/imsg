import type { ReactNode } from 'react'

type TextTone = 'accent' | 'ink' | 'muted' | 'surface'
type TextSize = 'sm' | 'xs'

const TONE_CLASS: Record<TextTone, string> = {
  accent: 'text-accent',
  ink: 'text-ink',
  muted: 'text-muted',
  surface: 'text-surface',
}

const SIZE_CLASS: Record<TextSize, string> = {
  sm: 'text-sm',
  xs: 'text-xs',
}

// `h1`/`h2` always render bold, regardless of `size` — headings read as headings everywhere
// They're used (SettingsTemplate's section labels, DeviceSetupTemplate's screen title) rather
// Than each call site opting in individually.
const HEADING_CLASS: Record<'h1' | 'h2', string> = {
  h1: 'text-base font-semibold',
  h2: 'text-sm font-semibold',
}

interface TextProps {
  children: ReactNode
  // 'span' for text nested inside a `<button>` (e.g. DeviceListItem) — `<p>` isn't valid
  // Phrasing content there. 'h1'/'h2' for section/screen headings (e.g. SettingsTemplate).
  as?: 'p' | 'span' | 'h1' | 'h2'
  size?: TextSize
  tone?: TextTone
}

const Text = ({ children, as: Tag = 'p', size = 'sm', tone = 'ink' }: TextProps): React.JSX.Element => {
  let sizeClass = SIZE_CLASS[size]
  if (Tag === 'h1' || Tag === 'h2') {
    sizeClass = HEADING_CLASS[Tag]
  }
  return <Tag className={`${sizeClass} ${TONE_CLASS[tone]}`}>{children}</Tag>
}

export default Text
