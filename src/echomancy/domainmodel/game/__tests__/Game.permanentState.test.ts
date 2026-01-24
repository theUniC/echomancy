import { describe, expect, test } from "vitest"
import {
  addArtifactToBattlefield,
  addCreatureToBattlefield,
  addEnchantmentToBattlefield,
  createStartedGame,
  createTestArtifact,
  createTestCreature,
  createTestEnchantment,
  createTestLand,
} from "./helpers"

/**
 * Tests for PermanentState refactor (Phase 2)
 *
 * Tests that Game.ts correctly uses PermanentState for ALL permanents,
 * not just creatures.
 */

describe("Game - PermanentState (Phase 2)", () => {
  describe("getPermanentState()", () => {
    test("returns permanent state for creature", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id)
      addCreatureToBattlefield(game, player1.id, creature)

      const state = game.getPermanentState(creature.instanceId)

      expect(state).toBeDefined()
      expect(state.isTapped).toBe(false)
      expect(state.creatureState).toBeDefined()
    })

    test("returns permanent state for artifact", () => {
      const { game, player1 } = createStartedGame()
      const artifact = createTestArtifact(player1.id)
      addArtifactToBattlefield(game, player1.id, artifact)

      const state = game.getPermanentState(artifact.instanceId)

      expect(state).toBeDefined()
      expect(state.isTapped).toBe(false)
      expect(state.creatureState).toBeUndefined()
    })

    test("returns permanent state for enchantment", () => {
      const { game, player1 } = createStartedGame()
      const enchantment = createTestEnchantment(player1.id)
      addEnchantmentToBattlefield(game, player1.id, enchantment)

      const state = game.getPermanentState(enchantment.instanceId)

      expect(state).toBeDefined()
      expect(state.isTapped).toBe(false)
      expect(state.creatureState).toBeUndefined()
    })

    test("throws PermanentNotFoundError if permanent doesn't exist", () => {
      const { game } = createStartedGame()

      expect(() => game.getPermanentState("nonexistent-id")).toThrow()
    })
  })

  describe("tapPermanent() - universal tapping", () => {
    test("can tap a creature", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id)
      addCreatureToBattlefield(game, player1.id, creature)

      game.tapPermanent(creature.instanceId)

      const state = game.getPermanentState(creature.instanceId)
      expect(state.isTapped).toBe(true)
    })

    test("can tap an artifact", () => {
      const { game, player1 } = createStartedGame()
      const artifact = createTestArtifact(player1.id)
      addArtifactToBattlefield(game, player1.id, artifact)

      game.tapPermanent(artifact.instanceId)

      const state = game.getPermanentState(artifact.instanceId)
      expect(state.isTapped).toBe(true)
    })

    test("can tap an enchantment (rare but legal)", () => {
      const { game, player1 } = createStartedGame()
      const enchantment = createTestEnchantment(player1.id)
      addEnchantmentToBattlefield(game, player1.id, enchantment)

      game.tapPermanent(enchantment.instanceId)

      const state = game.getPermanentState(enchantment.instanceId)
      expect(state.isTapped).toBe(true)
    })

    test("can tap a land", () => {
      const { game, player1 } = createStartedGame()
      const land = createTestLand(player1.id)
      // Directly use enterBattlefield for lands in tests
      game.enterBattlefield(land, player1.id)

      game.tapPermanent(land.instanceId)

      const state = game.getPermanentState(land.instanceId)
      expect(state.isTapped).toBe(true)
    })

    test("throws PermanentNotFoundError for nonexistent permanent", () => {
      const { game } = createStartedGame()

      expect(() => game.tapPermanent("nonexistent-id")).toThrow()
    })
  })

  describe("untapPermanent() - universal untapping", () => {
    test("can untap a creature", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id)
      addCreatureToBattlefield(game, player1.id, creature)
      game.tapPermanent(creature.instanceId)

      game.untapPermanent(creature.instanceId)

      const state = game.getPermanentState(creature.instanceId)
      expect(state.isTapped).toBe(false)
    })

    test("can untap an artifact", () => {
      const { game, player1 } = createStartedGame()
      const artifact = createTestArtifact(player1.id)
      addArtifactToBattlefield(game, player1.id, artifact)
      game.tapPermanent(artifact.instanceId)

      game.untapPermanent(artifact.instanceId)

      const state = game.getPermanentState(artifact.instanceId)
      expect(state.isTapped).toBe(false)
    })

    test("can untap a land", () => {
      const { game, player1 } = createStartedGame()
      const land = createTestLand(player1.id)
      game.enterBattlefield(land, player1.id)
      game.tapPermanent(land.instanceId)

      game.untapPermanent(land.instanceId)

      const state = game.getPermanentState(land.instanceId)
      expect(state.isTapped).toBe(false)
    })
  })

  describe("getCreatureState() - backward compatibility", () => {
    test("still works for creatures", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id, undefined, 2, 3)
      addCreatureToBattlefield(game, player1.id, creature)

      const state = game.getCreatureState(creature.instanceId)

      expect(state.basePower).toBe(2)
      expect(state.baseToughness).toBe(3)
      expect(state.isTapped).toBe(false)
    })

    test("throws when called on non-creature permanent", () => {
      const { game, player1 } = createStartedGame()
      const artifact = createTestArtifact(player1.id)
      addArtifactToBattlefield(game, player1.id, artifact)

      expect(() => game.getCreatureState(artifact.instanceId)).toThrow()
    })
  })

  describe("enterBattlefield() - initializes state for all permanents", () => {
    test("initializes creature state with creature sub-state", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id)

      game.enterBattlefield(creature, player1.id)

      const state = game.getPermanentState(creature.instanceId)
      expect(state.creatureState).toBeDefined()
      expect(state.creatureState?.hasSummoningSickness).toBe(true)
    })

    test("initializes artifact state without creature sub-state", () => {
      const { game, player1 } = createStartedGame()
      const artifact = createTestArtifact(player1.id)

      game.enterBattlefield(artifact, player1.id)

      const state = game.getPermanentState(artifact.instanceId)
      expect(state.creatureState).toBeUndefined()
    })

    test("initializes enchantment state without creature sub-state", () => {
      const { game, player1 } = createStartedGame()
      const enchantment = createTestEnchantment(player1.id)

      game.enterBattlefield(enchantment, player1.id)

      const state = game.getPermanentState(enchantment.instanceId)
      expect(state.creatureState).toBeUndefined()
    })

    test("initializes land state without creature sub-state", () => {
      const { game, player1 } = createStartedGame()
      const land = createTestLand(player1.id)

      game.enterBattlefield(land, player1.id)

      const state = game.getPermanentState(land.instanceId)
      expect(state.creatureState).toBeUndefined()
    })
  })
})
