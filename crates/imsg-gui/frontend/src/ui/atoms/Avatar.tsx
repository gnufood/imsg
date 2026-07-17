import { Contact } from 'lucide-react'

// Placeholder-only — no contact photos or names (contacts integration is deferred, see
// GUI_COMMANDS.md). Decorative: the address/name it stands in for is rendered as visible
// Text alongside it, so this stays aria-hidden.
const Avatar = (): React.JSX.Element => (
  <span className="grid size-8 shrink-0 place-items-center rounded-full border border-line bg-ink/5 text-muted">
    <Contact className="size-4" aria-hidden="true" />
  </span>
)

export default Avatar
