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
