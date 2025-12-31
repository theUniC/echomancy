import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import {
  GetGameStateQuery,
  GetGameStateQueryHandler,
} from "./GetGameStateQuery"

describe("GetGameStateQuery", () => {
  it("throws InvalidGameIdError when game ID is not a valid UUID", () => {
    const repository = new InMemoryGameRepository()
    const handler = new GetGameStateQueryHandler(repository)

    expect(() => {
      handler.handle(new GetGameStateQuery("invalid-id"))
    }).toThrow(InvalidGameIdError)
  })

  it("throws GameNotFoundError when game does not exist", () => {
    const repository = new InMemoryGameRepository()
    const handler = new GetGameStateQueryHandler(repository)
    const nonExistentGameId = uuidv4()

    expect(() => {
      handler.handle(new GetGameStateQuery(nonExistentGameId))
    }).toThrow(GameNotFoundError)
  })

  it("returns game state for unstarted game", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const game = Game.create(gameId)
    repository.add(game)

    const handler = new GetGameStateQueryHandler(repository)
    const state = handler.handle(new GetGameStateQuery(gameId))

    expect(state.gameId).toBe(gameId)
    // Unstarted game still has turn number 1 (initialized value)
    expect(state.currentTurnNumber).toBe(1)
    expect(state.turnOrder).toEqual([])
    expect(state.players).toEqual({})
  })

  it("returns game state for started game with correct structure", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)
    repository.add(game)

    const handler = new GetGameStateQueryHandler(repository)
    const state = handler.handle(new GetGameStateQuery(gameId))

    // Verify core structure
    expect(state.gameId).toBe(gameId)
    expect(state.currentTurnNumber).toBe(1)
    expect(state.currentPlayerId).toBe(player1.id)
    expect(state.turnOrder).toHaveLength(2)
    expect(state.turnOrder).toContain(player1.id)
    expect(state.turnOrder).toContain(player2.id)

    // Verify players exist with zones
    expect(state.players[player1.id]).toBeDefined()
    expect(state.players[player2.id]).toBeDefined()

    // Verify player state structure
    const playerState = state.players[player1.id]
    expect(playerState.lifeTotal).toBe(20)
    expect(playerState.manaPool).toEqual({
      W: 0,
      U: 0,
      B: 0,
      R: 0,
      G: 0,
      C: 0,
    })
    expect(playerState.zones.hand).toBeDefined()
    expect(playerState.zones.battlefield).toBeDefined()
    expect(playerState.zones.graveyard).toBeDefined()

    // Verify stack is empty
    expect(state.stack).toEqual([])
  })
})
