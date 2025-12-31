import { validate as isValidUUID } from "uuid"
import {
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

export class StartGameCommand {
  constructor(
    public gameId: string,
    public startingPlayerId: string,
  ) {}
}

export class StartGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId, startingPlayerId }: StartGameCommand) {
    // 1. Input validation
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    if (!isValidUUID(startingPlayerId)) {
      throw new InvalidPlayerIdError(startingPlayerId)
    }

    // 2. Existence check
    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    // 3. Domain logic (game.start validates player count, player exists, etc.)
    game.start(startingPlayerId)
  }
}
