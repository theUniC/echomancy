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

  it("populates each player's hand with 7 cards on game start", () => {
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
    if (!updatedGame) {
      throw new Error("Game not found after start")
    }

    const player1State = updatedGame.getPlayerState(player1Id)
    const player2State = updatedGame.getPlayerState(player2Id)

    expect(player1State.hand.cards).toHaveLength(7)
    expect(player2State.hand.cards).toHaveLength(7)
  })

  it("hand contains 2 lands and 5 creatures", () => {
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
    if (!updatedGame) {
      throw new Error("Game not found after start")
    }

    const player1State = updatedGame.getPlayerState(player1Id)
    const player2State = updatedGame.getPlayerState(player2Id)

    // Check player 1 composition
    const player1Lands = player1State.hand.cards.filter((card) =>
      card.definition.types.includes("LAND"),
    )
    const player1Creatures = player1State.hand.cards.filter((card) =>
      card.definition.types.includes("CREATURE"),
    )
    expect(player1Lands).toHaveLength(2)
    expect(player1Creatures).toHaveLength(5)

    // Check player 2 composition
    const player2Lands = player2State.hand.cards.filter((card) =>
      card.definition.types.includes("LAND"),
    )
    const player2Creatures = player2State.hand.cards.filter((card) =>
      card.definition.types.includes("CREATURE"),
    )
    expect(player2Lands).toHaveLength(2)
    expect(player2Creatures).toHaveLength(5)
  })

  it("each card has unique instanceId", () => {
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
    if (!updatedGame) {
      throw new Error("Game not found after start")
    }

    const player1State = updatedGame.getPlayerState(player1Id)
    const player2State = updatedGame.getPlayerState(player2Id)

    // Collect all card instance IDs
    const allInstanceIds = [
      ...player1State.hand.cards.map((c) => c.instanceId),
      ...player2State.hand.cards.map((c) => c.instanceId),
    ]

    // Check for uniqueness
    const uniqueIds = new Set(allInstanceIds)
    expect(uniqueIds.size).toBe(14) // 7 cards per player * 2 players
  })

  it("creatures have correct power/toughness and keywords", () => {
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
    if (!updatedGame) {
      throw new Error("Game not found after start")
    }

    const player1State = updatedGame.getPlayerState(player1Id)

    const creatures = player1State.hand.cards.filter((card) =>
      card.definition.types.includes("CREATURE"),
    )

    // Verify we have 5 creatures
    expect(creatures).toHaveLength(5)

    // All creatures should have defined power and toughness
    for (const creature of creatures) {
      expect(creature.definition.power).toBeDefined()
      expect(creature.definition.toughness).toBeDefined()
      expect(typeof creature.definition.power).toBe("number")
      expect(typeof creature.definition.toughness).toBe("number")
    }

    // Verify we have the expected creatures (by name)
    const creatureNames = creatures.map((c) => c.definition.name).sort()
    expect(creatureNames).toEqual([
      "Elite Vanguard",
      "Giant Spider",
      "Grizzly Bears",
      "Llanowar Elves",
      "Serra Angel",
    ])

    // Verify specific creatures have correct stats
    const grizzlyBears = creatures.find(
      (c) => c.definition.name === "Grizzly Bears",
    )
    expect(grizzlyBears?.definition.power).toBe(2)
    expect(grizzlyBears?.definition.toughness).toBe(2)

    const serraAngel = creatures.find(
      (c) => c.definition.name === "Serra Angel",
    )
    expect(serraAngel?.definition.power).toBe(4)
    expect(serraAngel?.definition.toughness).toBe(4)
    expect(serraAngel?.definition.staticAbilities).toContain("FLYING")
    expect(serraAngel?.definition.staticAbilities).toContain("VIGILANCE")

    const giantSpider = creatures.find(
      (c) => c.definition.name === "Giant Spider",
    )
    expect(giantSpider?.definition.power).toBe(2)
    expect(giantSpider?.definition.toughness).toBe(4)
    expect(giantSpider?.definition.staticAbilities).toContain("REACH")
  })
})
