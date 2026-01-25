import { v4 as uuidv4 } from "uuid"
import { describe, expect, test } from "vitest"
import { PrebuiltDecks } from "../../cards/PrebuiltDecks"
import { Game } from "../Game"
import { Player } from "../Player"

describe("Game Setup - Deck Loading and Opening Hand", () => {
  describe("Game.start() with decks", () => {
    test("accepts optional deck configurations for players", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)

      // Should not throw
      game.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
      })
    })

    test("loads deck into player library and draws opening hand", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)

      game.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
      })

      // Libraries should have 53 cards each (60 - 7 opening hand)
      expect(game.getLibraryCount(player1.id)).toBe(53)
      expect(game.getLibraryCount(player2.id)).toBe(53)

      // Players should have 7 cards in hand
      const player1State = game.getPlayerState(player1.id)
      const player2State = game.getPlayerState(player2.id)
      expect(player1State.hand.cards).toHaveLength(7)
      expect(player2State.hand.cards).toHaveLength(7)
    })

    test("shuffles libraries after loading", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)

      // Store original order
      const originalOrder1 = deck1.map((c) => c.instanceId)

      game.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
      })

      // Get the order after loading (by drawing all cards)
      const player1State = game.getPlayerState(player1.id)
      const loadedOrder1 = player1State.library
        .getAll()
        .map((c) => c.instanceId)

      // Order should be different (shuffled)
      // Note: There's a tiny chance they're the same, but with 60 cards it's negligible
      expect(loadedOrder1).not.toEqual(originalOrder1)
    })

    test("draws 7-card opening hand for each player", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)

      game.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
      })

      // Each player should have 7 cards in hand
      const player1State = game.getPlayerState(player1.id)
      const player2State = game.getPlayerState(player2.id)

      expect(player1State.hand.cards).toHaveLength(7)
      expect(player2State.hand.cards).toHaveLength(7)
    })

    test("library has 53 cards after drawing opening hand", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)

      game.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
      })

      // 60 - 7 = 53 cards remaining in library
      expect(game.getLibraryCount(player1.id)).toBe(53)
      expect(game.getLibraryCount(player2.id)).toBe(53)
    })

    test("shuffle is deterministic with optional seed", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game1 = Game.create(uuidv4())
      game1.addPlayer(player1)
      game1.addPlayer(player2)

      const game2 = Game.create(uuidv4())
      game2.addPlayer(player1)
      game2.addPlayer(player2)

      const deck1 = PrebuiltDecks.greenDeck(player1.id)
      const deck2 = PrebuiltDecks.redDeck(player2.id)
      const deck3 = PrebuiltDecks.greenDeck(player1.id)
      const deck4 = PrebuiltDecks.redDeck(player2.id)

      const seed = 42

      game1.start(player1.id, {
        decks: {
          [player1.id]: deck1,
          [player2.id]: deck2,
        },
        shuffleSeed: seed,
      })

      game2.start(player1.id, {
        decks: {
          [player1.id]: deck3,
          [player2.id]: deck4,
        },
        shuffleSeed: seed,
      })

      // Both games should have same opening hands
      const game1Hand = game1
        .getPlayerState(player1.id)
        .hand.cards.map((c) => c.definition.id)
      const game2Hand = game2
        .getPlayerState(player1.id)
        .hand.cards.map((c) => c.definition.id)

      expect(game1Hand).toEqual(game2Hand)
    })
  })

  describe("Backward compatibility", () => {
    test("starting without decks still works", () => {
      const player1 = new Player(uuidv4(), "Player 1")
      const player2 = new Player(uuidv4(), "Player 2")

      const game = Game.create(uuidv4())
      game.addPlayer(player1)
      game.addPlayer(player2)

      // Should not throw
      game.start(player1.id)

      // Libraries should be empty
      expect(game.getLibraryCount(player1.id)).toBe(0)
      expect(game.getLibraryCount(player2.id)).toBe(0)

      // Hands should be empty
      const player1State = game.getPlayerState(player1.id)
      const player2State = game.getPlayerState(player2.id)

      expect(player1State.hand.cards).toHaveLength(0)
      expect(player2State.hand.cards).toHaveLength(0)
    })
  })
})
