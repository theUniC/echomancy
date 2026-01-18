import type { Game } from "../Game"
import type { Specification } from "./Specification"

/**
 * Context for evaluating game action specifications.
 * Contains the game state and the player attempting the action.
 */
export type GameActionContext = {
  game: Game
  playerId: string
}

/**
 * Specification that checks if a player has priority.
 *
 * In Magic, priority determines who can take actions. The player
 * with priority can cast spells, activate abilities, or pass priority.
 *
 * @example
 * const spec = new HasPriority()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player can take priority-based actions
 * }
 */
export class HasPriority implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    return game.priorityPlayerId === playerId
  }
}
