type PriorityControlsProps = {
  canPassPriority: boolean
  onPass: () => void
  onEndTurn: () => void
  disabled?: boolean
}

/**
 * PriorityControls - Buttons for passing priority and ending turn
 *
 * Provides:
 * - "Pass" button to pass priority (ADVANCE_STEP)
 * - "End Turn" button to end the current turn (END_TURN)
 * - Disabled state when viewer doesn't have priority
 */
export function PriorityControls({
  canPassPriority,
  onPass,
  onEndTurn,
  disabled = false,
}: PriorityControlsProps) {
  const isDisabled = disabled || !canPassPriority

  const buttonStyle = (isDisabled: boolean) => ({
    padding: "8px 16px",
    marginRight: "8px",
    borderRadius: "4px",
    border: "1px solid #ccc",
    backgroundColor: isDisabled ? "#f5f5f5" : "#fff",
    color: isDisabled ? "#999" : "#333",
    fontSize: "14px",
    fontWeight: "600",
    cursor: isDisabled ? "not-allowed" : "pointer",
    opacity: isDisabled ? 0.6 : 1,
  })

  return (
    <div style={{ marginBottom: "16px" }}>
      <button
        type="button"
        onClick={onPass}
        disabled={isDisabled}
        style={buttonStyle(isDisabled)}
        aria-label="Pass priority"
      >
        Pass
      </button>
      <button
        type="button"
        onClick={onEndTurn}
        disabled={isDisabled}
        style={buttonStyle(isDisabled)}
        aria-label="End turn"
      >
        End Turn
      </button>
    </div>
  )
}
