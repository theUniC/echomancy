/**
 * OpponentHandCount - Displays opponent's hand card count
 *
 * Shows "Opponent's Hand: X card(s)" with proper singular/plural handling.
 * Renders as HTML/CSS above the canvas for simplicity.
 *
 * Styling:
 * - Background: #0D1117 (near-black)
 * - Text color: #B0B0B0 (gray)
 * - Font: Inter 16px semi-bold
 */

type OpponentHandCountProps = {
  count: number
}

export function OpponentHandCount({ count }: OpponentHandCountProps) {
  const cardText = count === 1 ? "card" : "cards"

  return (
    <div
      style={{
        backgroundColor: "#0D1117",
        color: "#B0B0B0",
        fontFamily: "Inter, sans-serif",
        fontSize: "16px",
        fontWeight: "600", // Semi-bold
        padding: "12px 0",
        textAlign: "center",
      }}
    >
      Opponent&apos;s Hand: {count} {cardText}
    </div>
  )
}
