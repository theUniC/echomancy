import { validate as isValidUUID } from "uuid"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import {
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
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

  handle(message: JoinGameCommand) {
    if (!isValidUUID(message.gameId)) {
      throw new InvalidGameIdError(message.gameId)
    }

    if (!isValidUUID(message.playerId)) {
      throw new InvalidPlayerIdError(message.playerId)
    }

    const game = this.gameRepository.byId(message.gameId)
    if (!game) {
      throw new GameNotFoundError(message.gameId)
    }

    const player = new Player(message.playerId, message.playerName)
    game.addPlayer(player)
  }
}
