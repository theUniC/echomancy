import { validate as isValidUUID } from "uuid"
import { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

export class CreateGameCommand {
  constructor(public id: string) {}
}

export class CreateGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle(message: CreateGameCommand) {
    if (!isValidUUID(message.id)) {
      throw new InvalidGameIdError(message.id)
    }

    const game = Game.create(message.id)
    this.gameRepository.add(game)
  }
}
