/**
 * CombatState Value Object
 *
 * Immutable value object that holds combat-related game state.
 * Tracks attacker declarations and blocker assignments.
 *
 * Properties:
 * - attackerIds: Set of creature IDs that are declared attackers
 * - blockerAssignments: Map of attackerId -> blockerId
 *
 * @example
 * const combat = CombatState.initial()
 * const withAttacker = combat.withAttacker("creature-1")
 * const withBlocker = withAttacker.withBlocker("creature-1", "creature-2")
 */

export type CombatStateSnapshot = {
  attackerIds: string[]
  blockerAssignments: Record<string, string>
}

export class CombatState {
  private readonly _attackerIds: ReadonlySet<string>
  private readonly _blockerAssignments: ReadonlyMap<string, string>

  private constructor(
    attackerIds: ReadonlySet<string>,
    blockerAssignments: ReadonlyMap<string, string>,
  ) {
    this._attackerIds = attackerIds
    this._blockerAssignments = blockerAssignments
  }

  /**
   * Creates initial empty combat state.
   */
  static initial(): CombatState {
    return new CombatState(new Set(), new Map())
  }

  /**
   * Creates CombatState from a snapshot.
   */
  static fromSnapshot(snapshot: CombatStateSnapshot): CombatState {
    return new CombatState(
      new Set(snapshot.attackerIds),
      new Map(Object.entries(snapshot.blockerAssignments)),
    )
  }

  /**
   * Returns a new CombatState with the creature declared as attacker.
   */
  withAttacker(creatureId: string): CombatState {
    const newAttackers = new Set(this._attackerIds)
    newAttackers.add(creatureId)
    return new CombatState(newAttackers, this._blockerAssignments)
  }

  /**
   * Returns a new CombatState without the specified attacker.
   */
  withoutAttacker(creatureId: string): CombatState {
    const newAttackers = new Set(this._attackerIds)
    newAttackers.delete(creatureId)

    // Also remove any blocker assignment for this attacker
    const newBlockers = new Map(this._blockerAssignments)
    newBlockers.delete(creatureId)

    return new CombatState(newAttackers, newBlockers)
  }

  /**
   * Returns a new CombatState with the blocker assigned to the attacker.
   */
  withBlocker(attackerId: string, blockerId: string): CombatState {
    const newBlockers = new Map(this._blockerAssignments)
    newBlockers.set(attackerId, blockerId)
    return new CombatState(this._attackerIds, newBlockers)
  }

  /**
   * Returns a new CombatState without the blocker assignment.
   */
  withoutBlocker(attackerId: string): CombatState {
    const newBlockers = new Map(this._blockerAssignments)
    newBlockers.delete(attackerId)
    return new CombatState(this._attackerIds, newBlockers)
  }

  /**
   * Checks if a creature is declared as attacker.
   */
  isAttacking(creatureId: string): boolean {
    return this._attackerIds.has(creatureId)
  }

  /**
   * Checks if an attacker is blocked.
   */
  isBlocked(attackerId: string): boolean {
    return this._blockerAssignments.has(attackerId)
  }

  /**
   * Checks if a creature is blocking any attacker.
   */
  isBlocking(creatureId: string): boolean {
    for (const blockerId of this._blockerAssignments.values()) {
      if (blockerId === creatureId) {
        return true
      }
    }
    return false
  }

  /**
   * Gets the blocker ID for an attacker, or null if unblocked.
   */
  getBlockerFor(attackerId: string): string | null {
    return this._blockerAssignments.get(attackerId) ?? null
  }

  /**
   * Gets the attacker ID that a creature is blocking, or null if not blocking.
   */
  getBlockedAttacker(blockerId: string): string | null {
    for (const [attackerId, bid] of this._blockerAssignments.entries()) {
      if (bid === blockerId) {
        return attackerId
      }
    }
    return null
  }

  /**
   * Gets all declared attacker IDs.
   */
  getAttackerIds(): ReadonlySet<string> {
    return this._attackerIds
  }

  /**
   * Gets the count of declared attackers.
   */
  get attackerCount(): number {
    return this._attackerIds.size
  }

  /**
   * Checks if there are any declared attackers.
   */
  hasAttackers(): boolean {
    return this._attackerIds.size > 0
  }

  /**
   * Gets all blocker assignments as entries.
   */
  getBlockerAssignments(): ReadonlyMap<string, string> {
    return this._blockerAssignments
  }

  /**
   * Clears all combat state (end of combat).
   */
  clear(): CombatState {
    return CombatState.initial()
  }

  /**
   * Returns a snapshot for persistence or comparison.
   */
  toSnapshot(): CombatStateSnapshot {
    const blockerAssignments: Record<string, string> = {}
    for (const [attackerId, blockerId] of this._blockerAssignments) {
      blockerAssignments[attackerId] = blockerId
    }

    return {
      attackerIds: Array.from(this._attackerIds),
      blockerAssignments,
    }
  }
}
