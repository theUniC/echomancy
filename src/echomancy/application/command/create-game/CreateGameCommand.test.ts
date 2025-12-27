import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import {
  CreateGameCommand,
  CreateGameCommandHandler,
} from "./CreateGameCommand"

describe("CreateGameCommand", () => {
  it("throws an exception when the provided ID is not a valid UUID", () => {
    const commandHandler = new CreateGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(new CreateGameCommand("invalid-id"))
    }).toThrow(InvalidGameIdError)
  })

  it("creates a new game given an ID", () => {
    const gameId = uuidv4()

    const gameRepository: GameRepository = new InMemoryGameRepository()
    const commandHandler = new CreateGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new CreateGameCommand(gameId))
    }).not.toThrowError()

    expect(gameRepository.byId(gameId)).toBeDefined()
  })
})
