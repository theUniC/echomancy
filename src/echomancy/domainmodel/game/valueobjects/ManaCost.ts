/**
 * ManaCost Value Object
 *
 * Represents the mana cost of a spell or ability.
 * Mana costs consist of:
 * - generic: Any type of mana (represented by numbers like "2")
 * - W, U, B, R, G: Colored mana (White, Blue, Black, Red, Green)
 * - C: Colorless mana (must be paid with colorless mana specifically)
 *
 * @example
 * // "2UU" -> { generic: 2, U: 2 }
 * // "1WW" -> { generic: 1, W: 2 }
 * // "BBB" -> { generic: 0, B: 3 }
 * // "4"   -> { generic: 4 }
 * // "2C"  -> { generic: 2, C: 1 }
 */

export type ManaCost = {
  /** Generic mana that can be paid with any color */
  generic: number
  /** White mana */
  W?: number
  /** Blue mana */
  U?: number
  /** Black mana */
  B?: number
  /** Red mana */
  R?: number
  /** Green mana */
  G?: number
  /** Colorless mana (must be paid with colorless specifically) */
  C?: number
}

/**
 * ManaCostParser
 *
 * Utility to parse mana cost strings into ManaCost objects.
 *
 * Supported formats:
 * - Numbers: "4" -> generic mana
 * - Color letters: "W", "U", "B", "R", "G" -> colored mana
 * - Colorless: "C" -> colorless mana
 * - Combinations: "2UU", "1WW", "BBB", etc.
 *
 * @example
 * ManaCostParser.parse("2UU") // { generic: 2, U: 2 }
 * ManaCostParser.parse("BBB") // { generic: 0, B: 3 }
 */
export class ManaCostParser {
  private static readonly VALID_COLORS = ["W", "U", "B", "R", "G", "C"]

  /**
   * Parses a mana cost string into a ManaCost object.
   *
   * @param costString - Mana cost in string format (e.g., "2UU", "BBB", "4")
   * @returns ManaCost object
   * @throws Error if the cost string contains invalid characters
   */
  static parse(costString: string): ManaCost {
    if (costString === "") {
      return { generic: 0 }
    }

    // Validate characters before parsing
    for (const char of costString) {
      const isDigit = char >= "0" && char <= "9"
      const isValidColor = ManaCostParser.VALID_COLORS.includes(char)
      if (!isDigit && !isValidColor) {
        throw new Error(`Invalid mana cost format: '${costString}'`)
      }
    }

    const cost: ManaCost = { generic: 0 }

    // Parse generic cost (leading numbers)
    let i = 0
    let genericString = ""
    while (
      i < costString.length &&
      costString[i] >= "0" &&
      costString[i] <= "9"
    ) {
      genericString += costString[i]
      i++
    }

    if (genericString) {
      cost.generic = Number.parseInt(genericString, 10)
    }

    // Parse colored mana
    for (; i < costString.length; i++) {
      const color = costString[i]
      if (ManaCostParser.VALID_COLORS.includes(color)) {
        const key = color as keyof Omit<ManaCost, "generic">
        cost[key] = (cost[key] || 0) + 1
      }
    }

    return cost
  }
}
