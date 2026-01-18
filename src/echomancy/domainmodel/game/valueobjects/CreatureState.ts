/**
 * CreatureState Value Object
 *
 * Immutable representation of a creature's state on the battlefield.
 * All operations return new instances.
 *
 * @example
 * const state = CreatureState.forCreature(card)
 * const tapped = state.withTapped(true)
 * const damaged = tapped.withDamage(3)
 */

import type { CardInstance } from "../../cards/CardInstance"

export type CounterType = "PLUS_ONE_PLUS_ONE"

export type CreatureStateSnapshot = {
  isTapped: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
  hasSummoningSickness: boolean
  basePower: number
  baseToughness: number
  counters: Map<CounterType, number>
  damageMarkedThisTurn: number
  blockingCreatureId: string | null
  blockedBy: string | null
}

export class CreatureState {
  readonly isTapped: boolean
  readonly isAttacking: boolean
  readonly hasAttackedThisTurn: boolean
  readonly hasSummoningSickness: boolean
  readonly basePower: number
  readonly baseToughness: number
  readonly counters: ReadonlyMap<CounterType, number>
  readonly damageMarkedThisTurn: number
  readonly blockingCreatureId: string | null
  readonly blockedBy: string | null

  private constructor(snapshot: CreatureStateSnapshot) {
    this.isTapped = snapshot.isTapped
    this.isAttacking = snapshot.isAttacking
    this.hasAttackedThisTurn = snapshot.hasAttackedThisTurn
    this.hasSummoningSickness = snapshot.hasSummoningSickness
    this.basePower = snapshot.basePower
    this.baseToughness = snapshot.baseToughness
    this.counters = new Map(snapshot.counters)
    this.damageMarkedThisTurn = snapshot.damageMarkedThisTurn
    this.blockingCreatureId = snapshot.blockingCreatureId
    this.blockedBy = snapshot.blockedBy
  }

  /**
   * Creates a new CreatureState for a creature entering the battlefield.
   */
  static forCreature(card: CardInstance): CreatureState {
    const power = card.definition.power ?? 0
    const toughness = card.definition.toughness ?? 1

    return new CreatureState({
      isTapped: false,
      isAttacking: false,
      hasAttackedThisTurn: false,
      hasSummoningSickness: true,
      basePower: power,
      baseToughness: toughness,
      counters: new Map(),
      damageMarkedThisTurn: 0,
      blockingCreatureId: null,
      blockedBy: null,
    })
  }

  /**
   * Creates a CreatureState from a snapshot.
   */
  static fromSnapshot(snapshot: CreatureStateSnapshot): CreatureState {
    return new CreatureState(snapshot)
  }

  withTapped(isTapped: boolean): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      isTapped,
    })
  }

  withAttacking(isAttacking: boolean): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      isAttacking,
    })
  }

  withHasAttackedThisTurn(hasAttackedThisTurn: boolean): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      hasAttackedThisTurn,
    })
  }

  withSummoningSickness(hasSummoningSickness: boolean): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      hasSummoningSickness,
    })
  }

  withDamage(damage: number): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      damageMarkedThisTurn: damage,
    })
  }

  withBlockingCreatureId(id: string | null): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      blockingCreatureId: id,
    })
  }

  withBlockedBy(id: string | null): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      blockedBy: id,
    })
  }

  addCounters(type: CounterType, amount: number): CreatureState {
    const newCounters = new Map(this.counters)
    const current = newCounters.get(type) ?? 0
    newCounters.set(type, current + amount)
    return new CreatureState({
      ...this.toSnapshot(),
      counters: newCounters,
    })
  }

  removeCounters(type: CounterType, amount: number): CreatureState {
    const newCounters = new Map(this.counters)
    const current = newCounters.get(type) ?? 0
    const newValue = Math.max(0, current - amount)
    if (newValue === 0) {
      newCounters.delete(type)
    } else {
      newCounters.set(type, newValue)
    }
    return new CreatureState({
      ...this.toSnapshot(),
      counters: newCounters,
    })
  }

  getCounters(type: CounterType): number {
    return this.counters.get(type) ?? 0
  }

  /**
   * Calculates current power including +1/+1 counters.
   */
  getCurrentPower(): number {
    const plusCounters = this.counters.get("PLUS_ONE_PLUS_ONE") ?? 0
    return this.basePower + plusCounters
  }

  /**
   * Calculates current toughness including +1/+1 counters.
   */
  getCurrentToughness(): number {
    const plusCounters = this.counters.get("PLUS_ONE_PLUS_ONE") ?? 0
    return this.baseToughness + plusCounters
  }

  /**
   * Checks if the creature has lethal damage.
   */
  hasLethalDamage(): boolean {
    return this.damageMarkedThisTurn >= this.getCurrentToughness()
  }

  /**
   * Resets combat-related state for a new turn.
   */
  resetForNewTurn(): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      isAttacking: false,
      hasAttackedThisTurn: false,
      damageMarkedThisTurn: 0,
      blockingCreatureId: null,
      blockedBy: null,
      hasSummoningSickness: false,
    })
  }

  /**
   * Clears damage marked this turn.
   */
  clearDamage(): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      damageMarkedThisTurn: 0,
    })
  }

  /**
   * Clears combat state (end of combat).
   */
  clearCombatState(): CreatureState {
    return new CreatureState({
      ...this.toSnapshot(),
      isAttacking: false,
      blockingCreatureId: null,
      blockedBy: null,
    })
  }

  /**
   * Returns a mutable snapshot for export.
   */
  toSnapshot(): CreatureStateSnapshot {
    return {
      isTapped: this.isTapped,
      isAttacking: this.isAttacking,
      hasAttackedThisTurn: this.hasAttackedThisTurn,
      hasSummoningSickness: this.hasSummoningSickness,
      basePower: this.basePower,
      baseToughness: this.baseToughness,
      counters: new Map(this.counters),
      damageMarkedThisTurn: this.damageMarkedThisTurn,
      blockingCreatureId: this.blockingCreatureId,
      blockedBy: this.blockedBy,
    }
  }

  /**
   * Exports to the format expected by GameStateExport.
   */
  toExport(): {
    isTapped: boolean
    isAttacking: boolean
    hasAttackedThisTurn: boolean
    hasSummoningSickness: boolean
    basePower: number
    baseToughness: number
    currentPower: number
    currentToughness: number
    counters: Record<string, number>
    damageMarkedThisTurn: number
    blockingCreatureId: string | null
    blockedBy: string | null
  } {
    const countersRecord: Record<string, number> = {}
    for (const [key, value] of this.counters) {
      countersRecord[key] = value
    }

    return {
      isTapped: this.isTapped,
      isAttacking: this.isAttacking,
      hasAttackedThisTurn: this.hasAttackedThisTurn,
      hasSummoningSickness: this.hasSummoningSickness,
      basePower: this.basePower,
      baseToughness: this.baseToughness,
      currentPower: this.getCurrentPower(),
      currentToughness: this.getCurrentToughness(),
      counters: countersRecord,
      damageMarkedThisTurn: this.damageMarkedThisTurn,
      blockingCreatureId: this.blockingCreatureId,
      blockedBy: this.blockedBy,
    }
  }
}
