import { describe, expect, test } from "vitest"
import { ManaCostParser } from "../valueobjects/ManaCost"
import { createGameInMainPhase, createTestSpell } from "./helpers"

describe("Game - Mana Payment Integration", () => {
  describe("CAST_SPELL with mana costs", () => {
    test("can cast spell with zero cost", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = { generic: 0 }

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)
    })

    test("can cast spell with sufficient mana", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("2U")

      // Add mana to pool using public API
      game.addMana(player1.id, "U", 1)
      game.addMana(player1.id, "R", 2)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)

      // Verify mana was spent
      const afterPool = game.getManaPool(player1.id)
      expect(
        afterPool.W +
          afterPool.U +
          afterPool.B +
          afterPool.R +
          afterPool.G +
          afterPool.C,
      ).toBe(0)
    })

    test("throws error when insufficient mana for colored cost", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("UU")

      // Add insufficient blue mana
      game.addMana(player1.id, "U", 1)
      game.addMana(player1.id, "R", 2)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: spell.instanceId,
          targets: [],
        })
      }).toThrow("Insufficient U mana")
    })

    test("throws error when insufficient total mana for generic cost", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("4")

      // Add insufficient mana
      game.addMana(player1.id, "R", 3)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: spell.instanceId,
          targets: [],
        })
      }).toThrow("Insufficient mana")
    })

    test("uses auto-pay logic correctly (colored first, then generic)", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("2U")

      // Add mana: U:1, R:1, G:1, C:1 (total 4)
      game.addMana(player1.id, "U", 1)
      game.addMana(player1.id, "R", 1)
      game.addMana(player1.id, "G", 1)
      game.addMana(player1.id, "C", 1)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      // Verify mana was spent: U:1 (colored), then generic:2 from C:1 + W:0 (then R is used)
      // Should leave G:1 remaining (priority order: colorless first, then W, U, B, R, G)
      const afterPool = game.getManaPool(player1.id)
      expect(afterPool.U).toBe(0)
      expect(afterPool.R).toBe(0)
      expect(afterPool.C).toBe(0)
      expect(afterPool.G).toBe(1)
    })

    test("can cast spell with colorless mana requirement", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("2C")

      // Add mana with colorless
      game.addMana(player1.id, "C", 1)
      game.addMana(player1.id, "R", 2)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)

      // Verify: C:1 (colorless req), then generic:2 from R:2
      const afterPool = game.getManaPool(player1.id)
      const total =
        afterPool.W +
        afterPool.U +
        afterPool.B +
        afterPool.R +
        afterPool.G +
        afterPool.C
      expect(total).toBe(0)
    })

    test("throws error when colorless requirement cannot be paid with colored mana", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("C")

      // Add only colored mana (no colorless)
      game.addMana(player1.id, "R", 2)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: spell.instanceId,
          targets: [],
        })
      }).toThrow("Insufficient C mana")
    })

    test("can cast multi-color spell with exact mana", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("1WU")

      // Add exact mana
      game.addMana(player1.id, "W", 1)
      game.addMana(player1.id, "U", 1)
      game.addMana(player1.id, "R", 1)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)

      // Verify: W:1, U:1 (colored), then generic:1 from R:1
      const afterPool = game.getManaPool(player1.id)
      const total =
        afterPool.W +
        afterPool.U +
        afterPool.B +
        afterPool.R +
        afterPool.G +
        afterPool.C
      expect(total).toBe(0)
    })

    test("spell without mana cost can be cast for free", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      // No mana cost set (undefined)

      game.getPlayerState(player1.id).hand.cards.push(spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)
    })

    test("mana payment happens before spell goes on stack", () => {
      const { game, player1 } = createGameInMainPhase()
      const spell = createTestSpell(player1.id)
      spell.definition.manaCost = ManaCostParser.parse("2U")

      // Add insufficient mana
      game.addMana(player1.id, "U", 1)

      const playerState = game.getPlayerState(player1.id)
      playerState.hand.cards.push(spell)

      // Should throw before spell goes on stack
      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: spell.instanceId,
          targets: [],
        })
      }).toThrow()

      // Stack should remain empty
      expect(game.getStack()).toHaveLength(0)
      // Card should remain in hand
      expect(playerState.hand.cards).toContainEqual(spell)
    })
  })
})
