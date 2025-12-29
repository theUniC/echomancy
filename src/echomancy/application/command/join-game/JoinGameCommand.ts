import { validate as isValidUUID } from "uuid"
import {
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"

export class JoinGameCommand {
  constructor(
    public gameId: string,
    public playerId: string,
    public playerName: string,
  ) {}
}

export class JoinGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId, playerId, playerName }: JoinGameCommand) {
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    if (!isValidUUID(playerId)) {
      throw new InvalidPlayerIdError(playerId)
    }

    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    const player = new Player(playerId, playerName)
    game.addPlayer(player)
  }
}
