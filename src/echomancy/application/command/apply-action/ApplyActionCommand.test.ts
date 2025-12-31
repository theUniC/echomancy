import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { createStartedGame } from "@/echomancy/domainmodel/game/__tests__/helpers"
import { Game } from "@/echomancy/domainmodel/game/Game"
import {
  GameNotFoundError,
  GameNotStartedError,
  InvalidPlayerActionError,
} from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { Step } from "@/echomancy/domainmodel/game/Steps"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import {
  ApplyActionCommand,
  ApplyActionCommandHandler,
} from "./ApplyActionCommand"

describe("ApplyActionCommand", () => {
  it("throws InvalidGameIdError for invalid game UUID", () => {
    const commandHandler = new ApplyActionCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(
        new ApplyActionCommand("invalid-game-id", {
          type: "ADVANCE_STEP",
          playerId: uuidv4(),
        }),
      )
    }).toThrow(InvalidGameIdError)
  })

  it("throws GameNotFoundError when game does not exist", () => {
    const commandHandler = new ApplyActionCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(
        new ApplyActionCommand(uuidv4(), {
          type: "ADVANCE_STEP",
          playerId: uuidv4(),
        }),
      )
    }).toThrow(GameNotFoundError)
  })

  it("throws GameNotStartedError when game has not been started", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const playerId = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(playerId, "Player 1"))
    game.addPlayer(new Player(uuidv4(), "Player 2"))
    gameRepository.add(game)

    const commandHandler = new ApplyActionCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(
        new ApplyActionCommand(gameId, {
          type: "ADVANCE_STEP",
          playerId,
        }),
      )
    }).toThrow(GameNotStartedError)
  })

  it("successfully applies ADVANCE_STEP action", () => {
    const gameRepository = new InMemoryGameRepository()
    const { game, player1 } = createStartedGame()
    gameRepository.add(game)

    const initialStep = game.currentStep

    const commandHandler = new ApplyActionCommandHandler(gameRepository)
    commandHandler.handle(
      new ApplyActionCommand(game.id, {
        type: "ADVANCE_STEP",
        playerId: player1.id,
      }),
    )

    expect(game.currentStep).not.toBe(initialStep)
  })

  it("successfully applies PASS_PRIORITY action", () => {
    const gameRepository = new InMemoryGameRepository()
    const { game, player1 } = createStartedGame()
    gameRepository.add(game)

    // Advance to a step where priority passing is meaningful
    while (game.currentStep !== Step.FIRST_MAIN) {
      game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    }

    const commandHandler = new ApplyActionCommandHandler(gameRepository)

    // PASS_PRIORITY should not throw - it should be a valid action
    expect(() => {
      commandHandler.handle(
        new ApplyActionCommand(game.id, {
          type: "PASS_PRIORITY",
          playerId: player1.id,
        }),
      )
    }).not.toThrow()
  })

  it("throws domain error for invalid action (wrong player)", () => {
    const gameRepository = new InMemoryGameRepository()
    const { game, player2 } = createStartedGame()
    gameRepository.add(game)

    const commandHandler = new ApplyActionCommandHandler(gameRepository)

    // player2 is not the active player, so they cannot advance the step
    expect(() => {
      commandHandler.handle(
        new ApplyActionCommand(game.id, {
          type: "ADVANCE_STEP",
          playerId: player2.id,
        }),
      )
    }).toThrow(InvalidPlayerActionError)
  })
})
