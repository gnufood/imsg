import { useEffect, useRef } from 'react'
import { motion } from 'motion/react'

interface ModalProps {
  children: React.ReactNode
  label: string
  onClose: () => void
}

const CARD_TRANSITION = { duration: 0.15 }
const CARD_HIDDEN = { opacity: 0, scale: 0.95 }
const CARD_VISIBLE = { opacity: 1, scale: 1 }

// Native <dialog> + showModal() — browser-native focus handling and Escape-to-close, rather
// Than reimplementing both by hand. `onClose` is wired to the dialog's own `close` event, so it
// Fires however the browser closed it. No backdrop-click-to-close: every current use is a
// Confirm dialog with explicit Cancel/Confirm buttons, and skipping it avoids an accidental
// Dismiss on what's usually a destructive-action confirm.
//
// The card itself (not the <dialog>) carries the scale/fade — Motion can't animate the native
// `::backdrop` pseudo-element, so that stays an instant CSS toggle; the caller needs to wrap its
// Conditional render in `AnimatePresence` for the exit half of this to actually play (otherwise
// It just unmounts instantly, same as before).
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
    <dialog ref={dialogRef} aria-label={label} className="m-auto border-0 bg-transparent p-0 backdrop:bg-ink/50">
      <motion.div
        animate={CARD_VISIBLE}
        className="w-full max-w-sm rounded-lg border border-line bg-surface p-4"
        exit={CARD_HIDDEN}
        initial={CARD_HIDDEN}
        transition={CARD_TRANSITION}
      >
        {children}
      </motion.div>
    </dialog>
  )
}

export default Modal
