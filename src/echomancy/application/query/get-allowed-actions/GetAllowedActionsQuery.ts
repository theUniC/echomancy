import { validate as isValidUUID } from "uuid"
import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

export type AllowedActionsResult = {
  playableLands: string[]
}

export class GetAllowedActionsQuery {
  constructor(
    public gameId: string,
    public playerId: string,
  ) {}
}

export class GetAllowedActionsQueryHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId, playerId }: GetAllowedActionsQuery): AllowedActionsResult {
    // 1. Input validation
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    // 2. Existence check
    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    // 3. Use domain's getAllowedActionsFor - single source of truth
    // This correctly checks: priority, empty stack, main phase, land limit
    const allowedActions = game.getAllowedActionsFor(playerId)

    if (!allowedActions.includes("PLAY_LAND")) {
      return { playableLands: [] }
    }

    // 4. Get land cards from player's hand
    const playerState = game.getPlayerState(playerId)
    const landCards = playerState.hand.cards.filter((card: CardInstance) =>
      card.definition.types.includes("LAND"),
    )

    return {
      playableLands: landCards.map((card: CardInstance) => card.instanceId),
    }
  }
}
