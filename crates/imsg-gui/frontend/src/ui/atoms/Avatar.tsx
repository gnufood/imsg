import { Contact } from 'lucide-react'

// Placeholder-only — PBAP exposes no contact photos, only display names. Decorative: the
// Address/name it stands in for is rendered as visible text alongside it, so this stays
// Aria-hidden.
const Avatar = (): React.JSX.Element => (
  <span className="grid size-8 shrink-0 place-items-center rounded-full border border-line bg-ink/5 text-muted">
    <Contact className="size-4" aria-hidden="true" />
  </span>
)

export default Avatar
