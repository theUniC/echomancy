import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { createStartedGame, createTestCreature } from "./helpers"

describe("Library - GameStateExport Integration", () => {
  test("empty library is exported", () => {
    const { game, player1 } = createStartedGame()

    const gameState = game.exportState()

    expect(gameState.players[player1.id]?.zones.library).toBeDefined()
    expect(gameState.players[player1.id]?.zones.library?.cards).toHaveLength(0)
  })

  test("library with cards exports card count", () => {
    const { game, player1 } = createStartedGame()

    const cards: CardInstance[] = [
      createTestCreature(player1.id, "card-1"),
      createTestCreature(player1.id, "card-2"),
      createTestCreature(player1.id, "card-3"),
    ]

    game.setDeck(player1.id, cards)

    const gameState = game.exportState()

    expect(gameState.players[player1.id]?.zones.library).toBeDefined()
    expect(gameState.players[player1.id]?.zones.library?.cards).toHaveLength(3)
  })

  test("library cards are exported with correct information", () => {
    const { game, player1 } = createStartedGame()

    const cards: CardInstance[] = [createTestCreature(player1.id, "card-1")]

    game.setDeck(player1.id, cards)

    const gameState = game.exportState()
    const libraryCards = gameState.players[player1.id]?.zones.library?.cards

    expect(libraryCards).toHaveLength(1)
    expect(libraryCards?.[0]?.instanceId).toBe("card-1")
    expect(libraryCards?.[0]?.ownerId).toBe(player1.id)
    expect(libraryCards?.[0]?.controllerId).toBe(player1.id)
  })

  test("library count decreases after draw", () => {
    const { game, player1 } = createStartedGame()

    const cards: CardInstance[] = [
      createTestCreature(player1.id, "card-1"),
      createTestCreature(player1.id, "card-2"),
    ]

    game.setDeck(player1.id, cards)

    // Initial state
    let gameState = game.exportState()
    expect(gameState.players[player1.id]?.zones.library?.cards).toHaveLength(2)

    // Draw one card
    game.drawCards(player1.id, 1)

    // After draw
    gameState = game.exportState()
    expect(gameState.players[player1.id]?.zones.library?.cards).toHaveLength(1)
  })
})
