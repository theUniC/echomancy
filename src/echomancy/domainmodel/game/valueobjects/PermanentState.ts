/**
 * PermanentState Value Object
 *
 * Immutable representation of a permanent's state on the battlefield.
 * Supports ALL permanent types (creatures, artifacts, enchantments, lands, planeswalkers).
 * All operations return new instances.
 *
 * @example
 * // Creature
 * const creatureState = PermanentState.forCreature(card)
 * const tapped = creatureState.withTapped(true)
 *
 * // Artifact
 * const artifactState = PermanentState.forNonCreature()
 * const charged = artifactState.addCounters("CHARGE", 3)
 */

import type { CardInstance } from "../../cards/CardInstance"

/**
 * Creature-specific state (combat, summoning sickness, P/T, damage).
 * This is optional and only present for creatures.
 */
export type CreatureSubState = {
  basePower: number
  baseToughness: number
  hasSummoningSickness: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
  damageMarkedThisTurn: number
  blockingCreatureId: string | null
  blockedBy: string | null
}

/**
 * Complete snapshot of permanent state (for export and reconstruction).
 */
export type PermanentStateSnapshot = {
  isTapped: boolean
  counters: Map<string, number>
  creatureState?: CreatureSubState
}

export class PermanentState {
  readonly isTapped: boolean
  readonly counters: ReadonlyMap<string, number>
  readonly creatureState?: Readonly<CreatureSubState>

  private constructor(snapshot: PermanentStateSnapshot) {
    this.isTapped = snapshot.isTapped
    this.counters = new Map(snapshot.counters)
    this.creatureState = snapshot.creatureState
      ? { ...snapshot.creatureState }
      : undefined
  }

  /**
   * Creates a new PermanentState for a creature entering the battlefield.
   */
  static forCreature(card: CardInstance): PermanentState {
    const power = card.definition.power ?? 0
    const toughness = card.definition.toughness ?? 1

    return new PermanentState({
      isTapped: false,
      counters: new Map(),
      creatureState: {
        basePower: power,
        baseToughness: toughness,
        hasSummoningSickness: true,
        isAttacking: false,
        hasAttackedThisTurn: false,
        damageMarkedThisTurn: 0,
        blockingCreatureId: null,
        blockedBy: null,
      },
    })
  }

  /**
   * Creates a new PermanentState for a non-creature permanent.
   */
  static forNonCreature(): PermanentState {
    return new PermanentState({
      isTapped: false,
      counters: new Map(),
      creatureState: undefined,
    })
  }

  /**
   * Creates a PermanentState from a snapshot.
   */
  static fromSnapshot(snapshot: PermanentStateSnapshot): PermanentState {
    return new PermanentState(snapshot)
  }

  // ============================================================================
  // Common Operations (All Permanents)
  // ============================================================================

  withTapped(isTapped: boolean): PermanentState {
    return new PermanentState({
      ...this.toSnapshot(),
      isTapped,
    })
  }

  addCounters(type: string, amount: number): PermanentState {
    const newCounters = new Map(this.counters)
    const current = newCounters.get(type) ?? 0
    newCounters.set(type, current + amount)
    return new PermanentState({
      ...this.toSnapshot(),
      counters: newCounters,
    })
  }

  removeCounters(type: string, amount: number): PermanentState {
    const newCounters = new Map(this.counters)
    const current = newCounters.get(type) ?? 0
    const newValue = Math.max(0, current - amount)
    if (newValue === 0) {
      newCounters.delete(type)
    } else {
      newCounters.set(type, newValue)
    }
    return new PermanentState({
      ...this.toSnapshot(),
      counters: newCounters,
    })
  }

  getCounters(type: string): number {
    return this.counters.get(type) ?? 0
  }

  // ============================================================================
  // Creature-Specific Operations
  // ============================================================================

  private requireCreatureState(): CreatureSubState {
    if (!this.creatureState) {
      throw new Error(
        "Cannot use creature-specific operation on non-creature permanent",
      )
    }
    return this.creatureState
  }

  withAttacking(isAttacking: boolean): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        isAttacking,
      },
    })
  }

  withHasAttackedThisTurn(hasAttackedThisTurn: boolean): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        hasAttackedThisTurn,
      },
    })
  }

  withSummoningSickness(hasSummoningSickness: boolean): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        hasSummoningSickness,
      },
    })
  }

  withDamage(damage: number): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        damageMarkedThisTurn: damage,
      },
    })
  }

  withBlockingCreatureId(id: string | null): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        blockingCreatureId: id,
      },
    })
  }

  withBlockedBy(id: string | null): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        blockedBy: id,
      },
    })
  }

  /**
   * Calculates current power including +1/+1 counters.
   */
  getCurrentPower(): number {
    const creatureState = this.requireCreatureState()
    const plusCounters = this.counters.get("PLUS_ONE_PLUS_ONE") ?? 0
    return creatureState.basePower + plusCounters
  }

  /**
   * Calculates current toughness including +1/+1 counters.
   */
  getCurrentToughness(): number {
    const creatureState = this.requireCreatureState()
    const plusCounters = this.counters.get("PLUS_ONE_PLUS_ONE") ?? 0
    return creatureState.baseToughness + plusCounters
  }

  /**
   * Checks if the creature has lethal damage.
   */
  hasLethalDamage(): boolean {
    const creatureState = this.requireCreatureState()
    return creatureState.damageMarkedThisTurn >= this.getCurrentToughness()
  }

  /**
   * Resets combat-related state for a new turn.
   */
  resetForNewTurn(): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        isAttacking: false,
        hasAttackedThisTurn: false,
        damageMarkedThisTurn: 0,
        blockingCreatureId: null,
        blockedBy: null,
        hasSummoningSickness: false,
      },
    })
  }

  /**
   * Clears damage marked this turn.
   */
  clearDamage(): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        damageMarkedThisTurn: 0,
      },
    })
  }

  /**
   * Clears combat state (end of combat).
   */
  clearCombatState(): PermanentState {
    const creatureState = this.requireCreatureState()
    return new PermanentState({
      ...this.toSnapshot(),
      creatureState: {
        ...creatureState,
        isAttacking: false,
        blockingCreatureId: null,
        blockedBy: null,
      },
    })
  }

  // ============================================================================
  // Export/Snapshot
  // ============================================================================

  /**
   * Returns a mutable snapshot for export and reconstruction.
   */
  toSnapshot(): PermanentStateSnapshot {
    return {
      isTapped: this.isTapped,
      counters: new Map(this.counters),
      creatureState: this.creatureState ? { ...this.creatureState } : undefined,
    }
  }
}
