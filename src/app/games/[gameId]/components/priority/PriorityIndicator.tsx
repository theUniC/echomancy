type PriorityIndicatorProps = {
  hasPriority: boolean
}

/**
 * PriorityIndicator - Shows who currently has priority
 *
 * Visual indicator that clearly distinguishes between:
 * - Viewer has priority (active state)
 * - Opponent has priority (waiting state)
 */
export function PriorityIndicator({ hasPriority }: PriorityIndicatorProps) {
  return (
    <div
      style={{
        padding: "8px 16px",
        borderRadius: "4px",
        border: "2px solid",
        borderColor: hasPriority ? "#4CAF50" : "#999",
        backgroundColor: hasPriority ? "#E8F5E9" : "#f5f5f5",
        color: hasPriority ? "#2E7D32" : "#666",
        fontWeight: "bold",
        fontSize: "14px",
        textAlign: "center",
        marginBottom: "8px",
      }}
    >
      {hasPriority ? "Your Priority" : "Opponent's Priority"}
    </div>
  )
}
