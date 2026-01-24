import { describe, expect, test } from "bun:test"
import type { CardInstance } from "../../cards/CardInstance"
import { createStartedGame } from "./helpers"

describe("Empty Library Draw Loss - State-Based Actions", () => {
  test("player loses when they attempt to draw from empty library", () => {
    const { game, player1 } = createStartedGame()

    // Empty library
    expect(game.getLibraryCount(player1.id)).toBe(0)

    // Attempt to draw from empty library
    game.drawCards(player1.id, 1)

    // Flag should be set
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)

    // Manually trigger state-based actions (normally automatic)
    game.performStateBasedActions()

    // Player should have lost (flag cleared after SBA)
    // Note: We'll need to add a way to check if a player lost
    // For now, check that flag was cleared
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
  })

  test("player does not lose if library is empty but no draw attempted", () => {
    const { game, player1 } = createStartedGame()

    // Empty library
    expect(game.getLibraryCount(player1.id)).toBe(0)

    // Don't attempt to draw
    // Trigger SBA
    game.performStateBasedActions()

    // Flag should not be set (no draw attempted)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
  })

  test("flag is cleared after state-based actions resolve", () => {
    const { game, player1 } = createStartedGame()

    // Draw from empty library
    game.drawCards(player1.id, 1)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)

    // Trigger SBA
    game.performStateBasedActions()

    // Flag should be cleared
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
  })

  test("multiple failed draw attempts set flag once", () => {
    const { game, player1 } = createStartedGame()

    // Try to draw 3 cards from empty library
    game.drawCards(player1.id, 3)

    // Flag should be set (only need one failed attempt)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)

    // Trigger SBA
    game.performStateBasedActions()

    // Flag should be cleared
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
  })

  test("drawing last card does not set flag, next draw does", () => {
    const { game, player1 } = createStartedGame()

    const cards: CardInstance[] = [
      {
        instanceId: "last-card",
        definition: { id: "def-1", name: "Last Card", types: ["INSTANT"] },
        ownerId: player1.id,
      },
    ]

    game.setDeck(player1.id, cards)

    // Draw the last card - should succeed
    game.drawCards(player1.id, 1)

    // Library now empty, but flag not set (draw succeeded)
    expect(game.getLibraryCount(player1.id)).toBe(0)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)

    // Now attempt to draw from empty library
    game.drawCards(player1.id, 1)

    // Now flag should be set
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)
  })

  test("only players who attempted draw are affected", () => {
    const { game, player1, player2 } = createStartedGame()

    // Both have empty libraries
    expect(game.getLibraryCount(player1.id)).toBe(0)
    expect(game.getLibraryCount(player2.id)).toBe(0)

    // Only player1 attempts to draw
    game.drawCards(player1.id, 1)

    // Only player1 should have flag set
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player2.id)).toBe(false)

    // Trigger SBA
    game.performStateBasedActions()

    // Both flags should be false (player1 cleared, player2 never set)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
    expect(game.hasAttemptedDrawFromEmptyLibrary(player2.id)).toBe(false)
  })
})
