import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { createStartedGame, createTestCreature } from "./helpers"

describe("Library - PlayerState Integration", () => {
  describe("addPlayer() initializes library", () => {
    test("player starts with empty library", () => {
      const { game, player1 } = createStartedGame()

      const libraryCount = game.getLibraryCount(player1.id)
      expect(libraryCount).toBe(0)
    })
  })

  describe("setDeck() before game start", () => {
    test("sets player deck with cards in order", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
        createTestCreature(player1.id, "card-3"),
      ]

      game.setDeck(player1.id, cards)

      expect(game.getLibraryCount(player1.id)).toBe(3)
    })

    test("throws error if player does not exist", () => {
      const { game } = createStartedGame()

      expect(() => {
        game.setDeck("non-existent-player", [])
      }).toThrow()
    })
  })

  describe("getLibraryCount() query", () => {
    test("returns correct library count", () => {
      const { game, player1 } = createStartedGame()

      const cards: CardInstance[] = [
        createTestCreature(player1.id, "card-1"),
        createTestCreature(player1.id, "card-2"),
      ]

      game.setDeck(player1.id, cards)

      expect(game.getLibraryCount(player1.id)).toBe(2)
    })

    test("throws error if player does not exist", () => {
      const { game } = createStartedGame()

      expect(() => {
        game.getLibraryCount("non-existent-player")
      }).toThrow()
    })
  })
})
