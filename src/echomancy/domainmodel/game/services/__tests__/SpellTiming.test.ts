import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../../../cards/CardInstance"
import { createStartedGame } from "../../__tests__/helpers"
import { Step } from "../../Steps"
import { SpellTimingService } from "../SpellTiming"

describe("SpellTimingService", () => {
  describe("isSorcerySpeed", () => {
    test("returns true for sorceries", () => {
      const card: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(true)
    })

    test("returns true for creatures without Flash", () => {
      const card: CardInstance = {
        instanceId: "test-creature",
        definition: {
          id: "test-creature",
          name: "Test Creature",
          types: ["CREATURE"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(true)
    })

    test("returns false for creatures with Flash", () => {
      const card: CardInstance = {
        instanceId: "flash-creature",
        definition: {
          id: "flash-creature",
          name: "Flash Creature",
          types: ["CREATURE"],
          staticAbilities: ["FLASH"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(false)
    })

    test("returns false for instants", () => {
      const card: CardInstance = {
        instanceId: "test-instant",
        definition: {
          id: "test-instant",
          name: "Test Instant",
          types: ["INSTANT"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(false)
    })

    test("returns true for artifacts without Flash", () => {
      const card: CardInstance = {
        instanceId: "test-artifact",
        definition: {
          id: "test-artifact",
          name: "Test Artifact",
          types: ["ARTIFACT"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(true)
    })

    test("returns false for artifacts with Flash", () => {
      const card: CardInstance = {
        instanceId: "flash-artifact",
        definition: {
          id: "flash-artifact",
          name: "Flash Artifact",
          types: ["ARTIFACT"],
          staticAbilities: ["FLASH"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(false)
    })

    test("returns true for enchantments without Flash", () => {
      const card: CardInstance = {
        instanceId: "test-enchantment",
        definition: {
          id: "test-enchantment",
          name: "Test Enchantment",
          types: ["ENCHANTMENT"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(true)
    })

    test("returns false for enchantments with Flash", () => {
      const card: CardInstance = {
        instanceId: "flash-enchantment",
        definition: {
          id: "flash-enchantment",
          name: "Flash Enchantment",
          types: ["ENCHANTMENT"],
          staticAbilities: ["FLASH"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(false)
    })

    test("returns true for planeswalkers", () => {
      const card: CardInstance = {
        instanceId: "test-planeswalker",
        definition: {
          id: "test-planeswalker",
          name: "Test Planeswalker",
          types: ["PLANESWALKER"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isSorcerySpeed(card)).toBe(true)
    })
  })

  describe("isInstantSpeed", () => {
    test("returns true for instants", () => {
      const card: CardInstance = {
        instanceId: "test-instant",
        definition: {
          id: "test-instant",
          name: "Test Instant",
          types: ["INSTANT"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isInstantSpeed(card)).toBe(true)
    })

    test("returns true for creatures with Flash", () => {
      const card: CardInstance = {
        instanceId: "flash-creature",
        definition: {
          id: "flash-creature",
          name: "Flash Creature",
          types: ["CREATURE"],
          staticAbilities: ["FLASH"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isInstantSpeed(card)).toBe(true)
    })

    test("returns false for creatures without Flash", () => {
      const card: CardInstance = {
        instanceId: "test-creature",
        definition: {
          id: "test-creature",
          name: "Test Creature",
          types: ["CREATURE"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isInstantSpeed(card)).toBe(false)
    })

    test("returns false for sorceries", () => {
      const card: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isInstantSpeed(card)).toBe(false)
    })

    test("returns true for artifacts with Flash", () => {
      const card: CardInstance = {
        instanceId: "flash-artifact",
        definition: {
          id: "flash-artifact",
          name: "Flash Artifact",
          types: ["ARTIFACT"],
          staticAbilities: ["FLASH"],
        },
        ownerId: "player1",
      }

      expect(SpellTimingService.isInstantSpeed(card)).toBe(true)
    })
  })

  describe("canCastAtCurrentTiming", () => {
    test("returns true for sorcery on own main phase with empty stack", () => {
      const { game, player1 } = createStartedGame()

      // Advance to main phase
      while (game.currentStep !== Step.FIRST_MAIN) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }

      const sorcery: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: player1.id,
      }

      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player1.id, sorcery),
      ).toBe(true)
    })

    test("returns false for sorcery on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to opponent's main phase
      while (game.currentStep !== Step.FIRST_MAIN) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }
      expect(game.currentPlayerId).toBe(player1.id) // Confirm player1's turn

      const sorcery: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: player2.id,
      }

      // Player2 tries to cast sorcery during player1's turn
      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player2.id, sorcery),
      ).toBe(false)
    })

    test("returns false for sorcery outside main phase", () => {
      const { game, player1 } = createStartedGame()

      // Advance to combat phase
      while (game.currentStep !== Step.BEGINNING_OF_COMBAT) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }

      const sorcery: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: player1.id,
      }

      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player1.id, sorcery),
      ).toBe(false)
    })

    test("returns false for sorcery with non-empty stack", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to main phase
      while (game.currentStep !== Step.FIRST_MAIN) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }

      // Add an instant to player's hand and cast it
      const instant: CardInstance = {
        instanceId: "test-instant",
        definition: {
          id: "test-instant",
          name: "Test Instant",
          types: ["INSTANT"],
        },
        ownerId: player1.id,
      }
      const playerState = game.getPlayerState(player1.id)
      playerState.hand.cards.push(instant)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Stack is now non-empty
      expect(game.getStack()).toHaveLength(1)

      const sorcery: CardInstance = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY"],
        },
        ownerId: player2.id,
      }

      // Player2 (with priority) tries to cast sorcery while stack is not empty
      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player2.id, sorcery),
      ).toBe(false)
    })

    test("returns true for instant anytime player has priority", () => {
      const { game, player1 } = createStartedGame()

      // Advance to any phase
      while (game.currentStep !== Step.BEGINNING_OF_COMBAT) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }

      const instant: CardInstance = {
        instanceId: "test-instant",
        definition: {
          id: "test-instant",
          name: "Test Instant",
          types: ["INSTANT"],
        },
        ownerId: player1.id,
      }

      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player1.id, instant),
      ).toBe(true)
    })

    test("returns true for Flash creature on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to opponent's main phase
      while (game.currentStep !== Step.FIRST_MAIN) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }
      expect(game.currentPlayerId).toBe(player1.id) // Confirm player1's turn

      const flashCreature: CardInstance = {
        instanceId: "flash-creature",
        definition: {
          id: "flash-creature",
          name: "Flash Creature",
          types: ["CREATURE"],
          staticAbilities: ["FLASH"],
        },
        ownerId: player2.id,
      }

      // Give priority to player2 (happens when player1 passes)
      const instant: CardInstance = {
        instanceId: "setup-instant",
        definition: {
          id: "setup-instant",
          name: "Setup Instant",
          types: ["INSTANT"],
        },
        ownerId: player1.id,
      }
      const p1State = game.getPlayerState(player1.id)
      p1State.hand.cards.push(instant)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Now player2 has priority on player1's turn
      expect(
        SpellTimingService.canCastAtCurrentTiming(
          game,
          player2.id,
          flashCreature,
        ),
      ).toBe(true)
    })

    test("returns false for creature without Flash on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to opponent's main phase
      while (game.currentStep !== Step.FIRST_MAIN) {
        game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
      }
      expect(game.currentPlayerId).toBe(player1.id) // Confirm player1's turn

      const creature: CardInstance = {
        instanceId: "test-creature",
        definition: {
          id: "test-creature",
          name: "Test Creature",
          types: ["CREATURE"],
        },
        ownerId: player2.id,
      }

      // Player2 tries to cast creature during player1's turn
      expect(
        SpellTimingService.canCastAtCurrentTiming(game, player2.id, creature),
      ).toBe(false)
    })
  })
})
