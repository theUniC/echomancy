import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import {
  GameAlreadyStartedError,
  GameNotFoundError,
  InvalidPlayerCountError,
  InvalidPlayerIdError,
  InvalidStartingPlayerError,
} from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import { StartGameCommand, StartGameCommandHandler } from "./StartGameCommand"

describe("StartGameCommand", () => {
  it("throws an exception when the provided game ID is not a valid UUID", () => {
    const commandHandler = new StartGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(new StartGameCommand("invalid-game-id", uuidv4()))
    }).toThrow(InvalidGameIdError)
  })

  it("throws an exception when the provided starting player ID is not a valid UUID", () => {
    const commandHandler = new StartGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(new StartGameCommand(uuidv4(), "invalid-player-id"))
    }).toThrow(InvalidPlayerIdError)
  })

  it("throws an exception when the game does not exist", () => {
    const commandHandler = new StartGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(new StartGameCommand(uuidv4(), uuidv4()))
    }).toThrow(GameNotFoundError)
  })

  it("throws an exception when the game has less than 2 players", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const playerId = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(playerId, "Player 1"))
    gameRepository.add(game)

    const commandHandler = new StartGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new StartGameCommand(gameId, playerId))
    }).toThrow(InvalidPlayerCountError)
  })

  it("throws an exception when the starting player is not in the game", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1Id = uuidv4()
    const player2Id = uuidv4()
    const nonExistentPlayerId = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    gameRepository.add(game)

    const commandHandler = new StartGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new StartGameCommand(gameId, nonExistentPlayerId))
    }).toThrow(InvalidStartingPlayerError)
  })

  it("successfully starts a game with valid inputs", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1Id = uuidv4()
    const player2Id = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    gameRepository.add(game)

    const commandHandler = new StartGameCommandHandler(gameRepository)
    commandHandler.handle(new StartGameCommand(gameId, player1Id))

    const updatedGame = gameRepository.byId(gameId)
    expect(updatedGame?.currentPlayerId).toBe(player1Id)
  })

  it("throws an exception when trying to start a game that has already started", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1Id = uuidv4()
    const player2Id = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    game.start(player1Id)
    gameRepository.add(game)

    const commandHandler = new StartGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new StartGameCommand(gameId, player2Id))
    }).toThrow(GameAlreadyStartedError)
  })
})
