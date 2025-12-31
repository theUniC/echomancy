import { validate as isValidUUID } from "uuid"
import type { Actions } from "@/echomancy/domainmodel/game/GameActions"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

export class ApplyActionCommand {
  constructor(
    public gameId: string,
    public action: Actions,
  ) {}
}

export class ApplyActionCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId, action }: ApplyActionCommand) {
    // 1. Input validation
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    // 2. Existence check
    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    // 3. Domain logic (game.apply validates lifecycle and all game rules)
    game.apply(action)
  }
}
