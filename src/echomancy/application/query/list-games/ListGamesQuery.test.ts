import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import {
  type GameSummary,
  ListGamesQuery,
  ListGamesQueryHandler,
} from "./ListGamesQuery"

describe("ListGamesQuery", () => {
  it("returns empty array when no games exist", () => {
    const repository = new InMemoryGameRepository()
    const handler = new ListGamesQueryHandler(repository)

    const result = handler.handle(new ListGamesQuery())

    expect(result).toEqual([])
  })

  it("returns game summary for unstarted game with correct status", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Alice")
    const player2 = new Player(uuidv4(), "Bob")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    repository.add(game)

    const handler = new ListGamesQueryHandler(repository)
    const result = handler.handle(new ListGamesQuery())

    expect(result).toHaveLength(1)
    const summary = result[0] as GameSummary
    expect(summary.gameId).toBe(gameId)
    expect(summary.status).toBe("not_started")
    expect(summary.playerNames).toEqual(["Alice", "Bob"])
    expect(summary.turnNumber).toBeNull()
    expect(summary.currentPhase).toBeNull()
  })

  it("returns game summary for started game with in_progress status", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Alice")
    const player2 = new Player(uuidv4(), "Bob")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)
    repository.add(game)

    const handler = new ListGamesQueryHandler(repository)
    const result = handler.handle(new ListGamesQuery())

    expect(result).toHaveLength(1)
    const summary = result[0] as GameSummary
    expect(summary.gameId).toBe(gameId)
    expect(summary.status).toBe("in_progress")
    expect(summary.playerNames).toEqual(["Alice", "Bob"])
    expect(summary.turnNumber).toBe(1)
    expect(summary.currentPhase).toBe("UNTAP")
  })

  it("returns multiple games", () => {
    const repository = new InMemoryGameRepository()

    // Game 1 - unstarted
    const game1Id = uuidv4()
    const game1 = Game.create(game1Id)
    game1.addPlayer(new Player(uuidv4(), "Player1"))
    game1.addPlayer(new Player(uuidv4(), "Player2"))
    repository.add(game1)

    // Game 2 - started
    const game2Id = uuidv4()
    const game2 = Game.create(game2Id)
    const game2Player1 = new Player(uuidv4(), "Alice")
    game2.addPlayer(game2Player1)
    game2.addPlayer(new Player(uuidv4(), "Bob"))
    game2.start(game2Player1.id)
    repository.add(game2)

    const handler = new ListGamesQueryHandler(repository)
    const result = handler.handle(new ListGamesQuery())

    expect(result).toHaveLength(2)

    // Find each game in the result
    const game1Summary = result.find((g) => g.gameId === game1Id)
    const game2Summary = result.find((g) => g.gameId === game2Id)

    expect(game1Summary).toBeDefined()
    expect(game1Summary?.status).toBe("not_started")

    expect(game2Summary).toBeDefined()
    expect(game2Summary?.status).toBe("in_progress")
  })

  it("returns game with no players", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const game = Game.create(gameId)
    repository.add(game)

    const handler = new ListGamesQueryHandler(repository)
    const result = handler.handle(new ListGamesQuery())

    expect(result).toHaveLength(1)
    const summary = result[0] as GameSummary
    expect(summary.gameId).toBe(gameId)
    expect(summary.status).toBe("not_started")
    expect(summary.playerNames).toEqual([])
    expect(summary.turnNumber).toBeNull()
    expect(summary.currentPhase).toBeNull()
  })
})
