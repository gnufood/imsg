import { Contact } from 'lucide-react'

const Avatar = (): React.JSX.Element => (
  <span className="grid size-8 shrink-0 place-items-center rounded-full border border-line bg-ink/5 text-muted">
    <Contact className="size-4" aria-hidden="true" />
  </span>
)

export default Avatar
