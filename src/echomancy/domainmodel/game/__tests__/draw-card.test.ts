import { describe, expect, test } from "bun:test"
import type { CardInstance } from "../../cards/CardInstance"
import { Step } from "../Steps"
import { advanceToStep, createStartedGame, createTestCreature } from "./helpers"

describe("Draw Card Implementation", () => {
  describe("drawCards() implementation", () => {
    test("draws one card from library to hand", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
      ]

      game.setDeck(player1.id, cards)

      // Initial state
      expect(game.getLibraryCount(player1.id)).toBe(2)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(0)

      // Draw one card
      game.drawCards(player1.id, 1)

      // Verify library decreased
      expect(game.getLibraryCount(player1.id)).toBe(1)

      // Verify hand increased
      expect(game.getPlayerState(player1.id).hand.count()).toBe(1)

      // Verify correct card was drawn (top card)
      const handCards = game.getPlayerState(player1.id).hand.getAll()
      expect(handCards[0]?.instanceId).toBe("card-1")
    })

    test("draws multiple cards in sequence", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
        createTestCreature(player1.id, "card-3"),
      ]

      game.setDeck(player1.id, cards)

      // Draw 2 cards
      game.drawCards(player1.id, 2)

      expect(game.getLibraryCount(player1.id)).toBe(1)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(2)

      // Verify order (card-1, card-2)
      const handCards = game.getPlayerState(player1.id).hand.getAll()
      expect(handCards[0]?.instanceId).toBe("card-1")
      expect(handCards[1]?.instanceId).toBe("card-2")
    })

    test("sets flag when attempting to draw from empty library", () => {
      const { game, player1 } = createStartedGame()

      // Empty library
      expect(game.getLibraryCount(player1.id)).toBe(0)

      // Attempt to draw from empty library
      game.drawCards(player1.id, 1)

      // Should set "attempted draw from empty library" flag
      // This will be checked by State-Based Actions
      expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)

      // Library and hand remain unchanged
      expect(game.getLibraryCount(player1.id)).toBe(0)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(0)
    })

    test("emits ZONE_CHANGED event when card drawn", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [createTestCreature(player1.id, "card-1")]

      game.setDeck(player1.id, cards)

      let eventFired = false
      const testCreature = createTestCreature(player1.id, "test-trigger")
      testCreature.definition.triggers = [
        {
          eventType: "ZONE_CHANGED",
          condition: (_, event) => {
            eventFired = true
            return (
              event.card.instanceId === "card-1" &&
              event.fromZone === "LIBRARY" &&
              event.toZone === "HAND"
            )
          },
          effect: () => {},
        },
      ]

      // Add trigger source to battlefield to enable trigger
      game.enterBattlefield(testCreature, player1.id)

      // Draw card
      game.drawCards(player1.id, 1)

      // Verify event was emitted
      expect(eventFired).toBe(true)
    })
  })

  describe("automatic draw in DRAW step", () => {
    test("active player draws one card when entering DRAW step", () => {
      const { game, player1, player2 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
      ]

      game.setDeck(player1.id, cards)

      // Complete turn 1 (no draw on turn 1 for starting player)
      advanceToStep(game, Step.CLEANUP)
      game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

      // Now we're on player 2's turn, advance to their DRAW step or back to player 1 turn 2
      // Let's advance to player 1's turn 2 DRAW step
      advanceToStep(game, Step.CLEANUP) // Complete player2 turn
      game.apply({ type: "ADVANCE_STEP", playerId: player2.id }) // Start player1 turn 2
      advanceToStep(game, Step.DRAW) // Advance to DRAW step (turn 2)

      // Should have drawn 1 card automatically (turn 2)
      expect(game.getLibraryCount(player1.id)).toBe(1)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(1)
      expect(game.getPlayerState(player1.id).hand.getAll()[0]?.instanceId).toBe(
        "card-1",
      )
    })

    test("first turn player does not draw on their first draw step", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
      ]

      game.setDeck(player1.id, cards)

      // Current step is UNTAP (turn 1)
      expect(game.currentStep).toBe(Step.UNTAP)

      // Advance to DRAW step (first turn)
      advanceToStep(game, Step.DRAW)

      // First turn - player should NOT draw
      expect(game.getLibraryCount(player1.id)).toBe(2)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(0)
    })
  })

  describe("DRAW_CARD action", () => {
    test("DRAW_CARD action draws card for effect-triggered draws", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
      ]

      game.setDeck(player1.id, cards)

      // Move to main phase for valid action timing
      advanceToStep(game, Step.FIRST_MAIN)

      // Initial state
      expect(game.getLibraryCount(player1.id)).toBe(2)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(0)

      // Apply DRAW_CARD action
      game.apply({
        type: "DRAW_CARD",
        playerId: player1.id,
        amount: 1,
      })

      // Verify draw happened
      expect(game.getLibraryCount(player1.id)).toBe(1)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(1)
    })

    test("DRAW_CARD action can draw multiple cards", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
        createTestCreature(player1.id, "card-3"),
      ]

      game.setDeck(player1.id, cards)

      advanceToStep(game, Step.FIRST_MAIN)

      // Draw 3 cards with one action
      game.apply({
        type: "DRAW_CARD",
        playerId: player1.id,
        amount: 3,
      })

      expect(game.getLibraryCount(player1.id)).toBe(0)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(3)
    })
  })

  describe("edge cases", () => {
    test("drawing more cards than library size only draws available cards", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [createTestCreature(player1.id, "card-1")]

      game.setDeck(player1.id, cards)

      // Try to draw 5 cards when only 1 exists
      game.drawCards(player1.id, 5)

      // Should draw the 1 available card
      expect(game.getLibraryCount(player1.id)).toBe(0)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(1)

      // Should set flag for attempted draw from empty library (attempts 2-5)
      expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)
    })

    test("drawing zero cards does nothing", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [createTestCreature(player1.id, "card-1")]

      game.setDeck(player1.id, cards)

      game.drawCards(player1.id, 0)

      expect(game.getLibraryCount(player1.id)).toBe(1)
      expect(game.getPlayerState(player1.id).hand.count()).toBe(0)
    })
  })
})
