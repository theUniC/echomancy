/**
 * CombatResolution Domain Service
 *
 * Stateless service that calculates combat damage assignments.
 * Handles the COMBAT_DAMAGE step per MTG rules.
 *
 * Current implementation (MVP):
 * - Unblocked attackers deal damage to defending player
 * - Blocked attackers and blockers deal damage to each other
 * - All damage is dealt simultaneously
 *
 * MVP Limitations:
 * - First strike / Double strike not implemented
 * - Trample not implemented (blocked creature with removed blocker deals no damage)
 * - Deathtouch not implemented
 * - Multiple blockers per attacker not implemented
 * - Damage prevention not implemented
 *
 * @example
 * const assignments = CombatResolution.calculateDamageAssignments(game)
 * for (const assignment of assignments) {
 *   if (assignment.isPlayer) {
 *     game.dealDamageToPlayer(assignment.targetId, assignment.amount)
 *   } else {
 *     game.markDamageOnCreature(assignment.targetId, assignment.amount)
 *   }
 * }
 */

import type { Game } from "../Game"

/**
 * Represents a single damage assignment in combat.
 */
export type DamageAssignment = {
  /** The target of the damage (player ID or creature instance ID) */
  targetId: string
  /** The amount of damage to deal */
  amount: number
  /** True if target is a player, false if target is a creature */
  isPlayer: boolean
}

/**
 * Calculates all combat damage assignments for the current combat.
 *
 * This is a pure function that queries the game state and returns
 * a list of damage assignments. It does NOT modify the game.
 *
 * @param game - The game to calculate damage for
 * @returns Array of damage assignments to apply
 */
export function calculateDamageAssignments(game: Game): DamageAssignment[] {
  const damageAssignments: DamageAssignment[] = []
  const defendingPlayer = game.getOpponentOf(game.currentPlayerId)

  for (const [attackerId, attackerState] of game.getCreatureEntries()) {
    if (!attackerState.isAttacking) continue

    // Check if attacker still exists (may have been removed by instant/ability)
    const attackerPower = game.getCreaturePowerSafe(attackerId)
    if (attackerPower === null) continue // Attacker no longer exists, skip

    if (attackerState.blockedBy === null) {
      // Unblocked attacker: damage to defending player
      damageAssignments.push({
        targetId: defendingPlayer,
        amount: attackerPower,
        isPlayer: true,
      })
    } else {
      // Blocked attacker: damage to blocker
      const blockerId = attackerState.blockedBy

      // Check if blocker still exists
      const blockerPower = game.getCreaturePowerSafe(blockerId)
      if (blockerPower === null) {
        // Blocker disappeared - attacker deals no damage (combat trick scenario)
        // This is deterministic: if your blocker is removed, the attacker doesn't
        // suddenly deal damage to the player (MVP: no trample)
        continue
      }

      // Attacker damages blocker
      damageAssignments.push({
        targetId: blockerId,
        amount: attackerPower,
        isPlayer: false,
      })

      // Blocker damages attacker
      damageAssignments.push({
        targetId: attackerId,
        amount: blockerPower,
        isPlayer: false,
      })
    }
  }

  return damageAssignments
}

/**
 * CombatResolution namespace for organized service methods.
 */
export const CombatResolution = {
  calculateDamageAssignments,
} as const
