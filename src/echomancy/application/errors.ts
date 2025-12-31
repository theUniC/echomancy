/**
 * Re-exports domain errors for use by API routes.
 *
 * EXCEPTION: API routes normally cannot import from domainmodel/.
 * Errors are allowed as an exception because they define the API
 * error contract without exposing domain logic.
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
