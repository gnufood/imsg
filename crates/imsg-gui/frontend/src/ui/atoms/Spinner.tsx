import { LoaderCircle } from 'lucide-react'

const Spinner = (): React.JSX.Element => (
  <LoaderCircle className="size-7 animate-spin text-accent" aria-hidden="true" />
)

export default Spinner
