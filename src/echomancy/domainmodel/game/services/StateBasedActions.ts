/**
 * StateBasedActions Domain Service
 *
 * Stateless service that performs state-based actions per MTG rules.
 * State-based actions are checked after each priority pass and whenever
 * a player would receive priority.
 *
 * Current implementation (MVP):
 * - Destroy creatures with lethal damage (damage >= toughness)
 * - Destroy creatures with 0 or less toughness
 * - Player loses if they attempted to draw from empty library
 *
 * MVP Limitations:
 * - Indestructible not supported
 * - Player loss condition not checked (0 life)
 * - Legend rule not implemented
 *
 * @example
 * const creaturesToDestroy = StateBasedActions.findCreaturesToDestroy(game)
 * for (const id of creaturesToDestroy) {
 *   game.movePermanentToGraveyard(id, GraveyardReason.STATE_BASED)
 * }
 */

import type { Game } from "../Game"

/**
 * Finds all creatures that should be destroyed due to state-based actions.
 *
 * This is a pure function that queries the game state and returns
 * a list of creature IDs to destroy. It does NOT modify the game.
 *
 * @param game - The game to check
 * @returns Array of creature instance IDs that should be destroyed
 */
export function findCreaturesToDestroy(game: Game): string[] {
  const creaturesToDestroy: string[] = []

  for (const [creatureId, permanentState] of game.getCreatureEntries()) {
    // getCreatureEntries() only returns permanents with creature state
    if (!permanentState.creatureState) continue

    const currentToughness = game.getCurrentToughness(creatureId)

    // Check for lethal damage (damage marked >= toughness)
    if (permanentState.creatureState.damageMarkedThisTurn >= currentToughness) {
      creaturesToDestroy.push(creatureId)
      continue
    }

    // Check for 0 or less toughness
    if (currentToughness <= 0) {
      creaturesToDestroy.push(creatureId)
    }
  }

  return creaturesToDestroy
}

/**
 * Finds all players who should lose due to attempting to draw from empty library.
 *
 * Per MTG rules 121.4 and 704.5b: A player who attempted to draw a card from
 * an empty library loses the game.
 *
 * This is a pure function that queries the game state and returns
 * a list of player IDs who should lose. It does NOT modify the game.
 *
 * @param game - The game to check
 * @returns Array of player IDs who should lose
 */
export function findPlayersWhoAttemptedEmptyLibraryDraw(game: Game): string[] {
  const playersToLose: string[] = []

  for (const playerId of game.getPlayersInTurnOrder()) {
    if (game.hasAttemptedDrawFromEmptyLibrary(playerId)) {
      playersToLose.push(playerId)
    }
  }

  return playersToLose
}

/**
 * StateBasedActions namespace for organized service methods.
 * Using namespace pattern for future expansion and consistent API.
 */
export const StateBasedActions = {
  findCreaturesToDestroy,
  findPlayersWhoAttemptedEmptyLibraryDraw,
} as const
