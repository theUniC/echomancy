/**
 * Re-exports domain errors for use by API routes.
 *
 * API routes can only import from `application/` and `infrastructure/`.
 * This module provides access to domain errors without violating that rule.
 */

export {
  GameAlreadyStartedError,
  GameError,
  GameNotFoundError,
  GameNotStartedError,
  InvalidPlayerActionError,
  InvalidPlayerCountError,
  InvalidPlayerIdError,
  InvalidStartingPlayerError,
  PlayerNotFoundError,
} from "@/echomancy/domainmodel/game/GameErrors"

export { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
