import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'

interface SelectOption {
  label: string
  value: string
}

interface SelectProps {
  value: string
  onChange: (value: string) => void
  options: SelectOption[]
  disabled?: boolean
  // For explicit `<label htmlFor>` pairing, same convention as `TextInput` — `<button>` is a
  // Labelable element, so this still works.
  id?: string
}

interface CloseOnOutsideArgs {
  containerRef: React.RefObject<HTMLDivElement | null>
  open: boolean
  setOpen: (open: boolean) => void
}

// Closes on an outside pointerdown or Escape — the two ways a native `<select>` popup dismisses
// Itself for free, which this loses by not being one (see `Select`'s own comment for why).
const useCloseOnOutside = ({ containerRef, open, setOpen }: CloseOnOutsideArgs): void => {
  useEffect(() => {
    if (!open) {
      return
    }
    const handlePointerDown = (event: PointerEvent): void => {
      if (containerRef.current !== null && !containerRef.current.contains(event.target as Node)) {
        setOpen(false)
      }
    }
    const handleKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        setOpen(false)
      }
    }
    document.addEventListener('pointerdown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('pointerdown', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [containerRef, open, setOpen])
}

interface OptionRow {
  handleSelect: () => void
  label: string
  selected: boolean
  value: string
}

// Stable per-option handlers, built once per `options`/`value`/`onSelect` identity rather than a
// Fresh closure per render inside the JSX below (see `SegmentedControl`'s own `buildSegments`).
const buildOptionRows = (options: SelectOption[], value: string, onSelect: (optionValue: string) => void): OptionRow[] =>
  options.map((option) => ({
    handleSelect: () => {
      onSelect(option.value)
    },
    label: option.label,
    selected: option.value === value,
    value: option.value,
  }))

const renderOption = (row: OptionRow): React.JSX.Element => {
  let toneClass = 'text-ink'
  if (row.selected) {
    toneClass = 'bg-accent/10 text-accent'
  }
  return (
    <li key={row.value}>
      <button type="button" onClick={row.handleSelect} className={`w-full px-3.5 py-1 text-left text-sm ${toneClass} ${interactiveStyles.hover}`}>
        {row.label}
      </button>
    </li>
  )
}

// Bounded choice from a fixed option list — `TextInput`'s free-text sibling, for fields with a
// Known-valid set (e.g. ChannelOverridesForm's 1-30 RFCOMM channel) rather than parse-and-reject.
// Not a native `<select>`: this app's Linux/WebKitGTK Tauri webview doesn't clip the native
// Popup to the window — with 30 options it renders past the window edge with its own
// Uncontrollable scrollbar. This panel is regular DOM content, so its own `max-h`/
// `overflow-y-auto` actually bounds it. Plain buttons rather than `role="listbox"`/`"option"` —
// Oxlint's `jsx-a11y` config here prefers the native `select`/`option` elements over those ARIA
// Roles bolted onto `div`/`li`, so this leans on the buttons' own real interactive semantics
// Instead.
const Select = ({ value, onChange, options, disabled = false, id }: SelectProps): React.JSX.Element => {
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  useCloseOnOutside({ containerRef, open, setOpen })

  const toggleOpen = useCallback(() => {
    setOpen((current) => !current)
  }, [])

  const selectOption = useCallback(
    (optionValue: string) => {
      onChange(optionValue)
      setOpen(false)
    },
    [onChange],
  )

  const rows = useMemo(() => buildOptionRows(options, value, selectOption), [options, value, selectOption])
  const selectedLabel = options.find((option) => option.value === value)?.label ?? value

  return (
    <div ref={containerRef} className="relative w-full">
      <button
        type="button"
        id={id}
        disabled={disabled}
        aria-expanded={open}
        aria-haspopup="true"
        onClick={toggleOpen}
        className={`w-full rounded-md border border-line bg-transparent px-3.5 py-1.5 text-left text-sm text-ink ${interactiveStyles.focus} ${interactiveStyles.disabled}`}
      >
        {selectedLabel}
      </button>
      {/* `max-h-28` caps the panel at ~4 rows (28px/row) — keeps the editor short enough that
      Appearance, below Channels in the same column, stays visible without scrolling the page
      even with the panel open. `[scrollbar-width:none]`/`[&::-webkit-scrollbar]:hidden` keep it
      scrollable past that without a visible scrollbar. */}
      {open && (
        <ul className="absolute z-10 mt-1 max-h-28 w-full overflow-y-auto rounded-md border border-line bg-surface py-1 shadow-lg [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
          {rows.map((row) => renderOption(row))}
        </ul>
      )}
    </div>
  )
}

export default Select
