import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import {
  CannotAddPlayerAfterStartError,
  DuplicatePlayerError,
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import { JoinGameCommand, JoinGameCommandHandler } from "./JoinGameCommand"

describe("JoinGameCommand", () => {
  it("throws an exception when the provided game ID is not a valid UUID", () => {
    const commandHandler = new JoinGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(
        new JoinGameCommand("invalid-game-id", uuidv4(), "Player 1"),
      )
    }).toThrow(InvalidGameIdError)
  })

  it("throws an exception when the provided player ID is not a valid UUID", () => {
    const commandHandler = new JoinGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(
        new JoinGameCommand(uuidv4(), "invalid-player-id", "Player 1"),
      )
    }).toThrow(InvalidPlayerIdError)
  })

  it("throws an exception when the game does not exist", () => {
    const commandHandler = new JoinGameCommandHandler(
      new InMemoryGameRepository(),
    )

    expect(() => {
      commandHandler.handle(new JoinGameCommand(uuidv4(), uuidv4(), "Player 1"))
    }).toThrow(GameNotFoundError)
  })

  it("adds a player to an existing game", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const playerId = uuidv4()

    const game = Game.create(gameId)
    gameRepository.add(game)

    const commandHandler = new JoinGameCommandHandler(gameRepository)
    commandHandler.handle(new JoinGameCommand(gameId, playerId, "Player 1"))

    const updatedGame = gameRepository.byId(gameId)
    expect(updatedGame?.getPlayersInTurnOrder()).toContain(playerId)
  })

  it("throws an exception when the player already joined the game", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const playerId = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(playerId, "Player 1"))
    gameRepository.add(game)

    const commandHandler = new JoinGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new JoinGameCommand(gameId, playerId, "Player 1"))
    }).toThrow(DuplicatePlayerError)
  })

  it("throws an exception when joining a game that has already started", () => {
    const gameRepository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1Id = uuidv4()
    const player2Id = uuidv4()
    const player3Id = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    game.start(player1Id)
    gameRepository.add(game)

    const commandHandler = new JoinGameCommandHandler(gameRepository)

    expect(() => {
      commandHandler.handle(new JoinGameCommand(gameId, player3Id, "Player 3"))
    }).toThrow(CannotAddPlayerAfterStartError)
  })
})
