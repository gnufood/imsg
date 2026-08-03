import { useEffect, useRef } from 'react'

interface ModalProps {
  children: React.ReactNode
  label: string
  onClose: () => void
}

// Native <dialog> + showModal() — browser-native focus handling and Escape-to-close, rather
// Than reimplementing both by hand. `onClose` is wired to the dialog's own `close` event, so it
// Fires however the browser closed it. No backdrop-click-to-close: every current use is a
// Confirm dialog with explicit Cancel/Confirm buttons, and skipping it avoids an accidental
// Dismiss on what's usually a destructive-action confirm.
const Modal = ({ children, label, onClose }: ModalProps): React.JSX.Element => {
  const dialogRef = useRef<HTMLDialogElement>(null)

  useEffect(() => {
    dialogRef.current?.showModal()
  }, [])

  useEffect(() => {
    const dialog = dialogRef.current
    if (dialog === null) {
      return
    }
    dialog.addEventListener('close', onClose)
    return () => {
      dialog.removeEventListener('close', onClose)
    }
  }, [onClose])

  return (
    <dialog ref={dialogRef} aria-label={label} className="m-auto w-full max-w-sm rounded-lg border border-line bg-surface p-4 backdrop:bg-ink/50">
      {children}
    </dialog>
  )
}

export default Modal
