/**
 * ManaPool Value Object
 *
 * Immutable representation of a player's mana pool.
 * All operations return new instances.
 *
 * @example
 * const pool = ManaPool.empty()
 * const withRed = pool.add("R", 2)
 * const afterSpend = withRed.spend("R", 1)
 */

export type ManaColor = "W" | "U" | "B" | "R" | "G" | "C"

export type ManaPoolSnapshot = {
  W: number
  U: number
  B: number
  R: number
  G: number
  C: number
}

export class InsufficientManaError extends Error {
  constructor(color: ManaColor, requested: number, available: number) {
    super(
      `Insufficient ${color} mana: requested ${requested}, available ${available}`,
    )
    this.name = "InsufficientManaError"
  }
}

export class ManaPool {
  private readonly values: ManaPoolSnapshot

  private constructor(values: ManaPoolSnapshot) {
    this.values = Object.freeze({ ...values })
  }

  /**
   * Creates an empty mana pool with all colors at 0.
   */
  static empty(): ManaPool {
    return new ManaPool({ W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 })
  }

  /**
   * Creates a mana pool from a snapshot.
   */
  static fromSnapshot(snapshot: ManaPoolSnapshot): ManaPool {
    return new ManaPool(snapshot)
  }

  /**
   * Adds mana of the specified color.
   * @returns A new ManaPool instance with the added mana.
   */
  add(color: ManaColor, amount: number): ManaPool {
    if (amount < 0) {
      throw new Error("Cannot add negative mana")
    }
    return new ManaPool({
      ...this.values,
      [color]: this.values[color] + amount,
    })
  }

  /**
   * Spends mana of the specified color.
   * @throws InsufficientManaError if not enough mana is available.
   * @returns A new ManaPool instance with the spent mana removed.
   */
  spend(color: ManaColor, amount: number): ManaPool {
    if (amount < 0) {
      throw new Error("Cannot spend negative mana")
    }
    const available = this.values[color]
    if (available < amount) {
      throw new InsufficientManaError(color, amount, available)
    }
    return new ManaPool({
      ...this.values,
      [color]: available - amount,
    })
  }

  /**
   * Clears all mana from the pool.
   * @returns A new empty ManaPool instance.
   */
  clear(): ManaPool {
    return ManaPool.empty()
  }

  /**
   * Gets the amount of mana for a specific color.
   */
  get(color: ManaColor): number {
    return this.values[color]
  }

  /**
   * Checks if the pool is empty (all colors at 0).
   */
  isEmpty(): boolean {
    return Object.values(this.values).every((v) => v === 0)
  }

  /**
   * Checks if this pool equals another pool.
   */
  equals(other: ManaPool): boolean {
    return (
      this.values.W === other.values.W &&
      this.values.U === other.values.U &&
      this.values.B === other.values.B &&
      this.values.R === other.values.R &&
      this.values.G === other.values.G &&
      this.values.C === other.values.C
    )
  }

  /**
   * Returns a snapshot of the mana pool for export.
   */
  toSnapshot(): ManaPoolSnapshot {
    return { ...this.values }
  }

  /**
   * Gets the total amount of mana across all colors.
   */
  total(): number {
    return Object.values(this.values).reduce((sum, v) => sum + v, 0)
  }
}
