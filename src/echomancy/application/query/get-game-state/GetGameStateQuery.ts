import { validate as isValidUUID } from "uuid"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

export class GetGameStateQuery {
  constructor(public gameId: string) {}
}

export class GetGameStateQueryHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId }: GetGameStateQuery): GameStateExport {
    // 1. Input validation
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    // 2. Existence check
    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    // 3. Return data
    return game.exportState()
  }
}
