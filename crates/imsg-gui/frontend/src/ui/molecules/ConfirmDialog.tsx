import Button from '@/ui/atoms/Button.tsx'
import Modal from '@/ui/atoms/Modal.tsx'
import Text from '@/ui/atoms/Text.tsx'

interface ConfirmDialogProps {
  confirmLabel: string
  error: string | undefined
  message: string
  onCancel: () => void
  onConfirm: () => void
  pending?: boolean
  title: string
}

const ConfirmDialog = ({ confirmLabel, error, message, onCancel, onConfirm, pending = false, title }: ConfirmDialogProps): React.JSX.Element => (
  <Modal label={title} onClose={onCancel}>
    <div className="flex flex-col gap-4">
      <Text>{title}</Text>
      <Text size="xs" tone="muted">
        {message}
      </Text>
      {error !== undefined && (
        <Text size="xs" tone="muted">
          {error}
        </Text>
      )}
      <div className="flex justify-end gap-2">
        <Button disabled={pending} onClick={onCancel}>
          Cancel
        </Button>
        <Button disabled={pending} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </div>
    </div>
  </Modal>
)

export default ConfirmDialog
