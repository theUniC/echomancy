import { describe, expect, test, vi } from "vitest"
import {
  addCreatureToBattlefield,
  createStartedGame,
  createTestCreature,
} from "../../__tests__/helpers"
import { GameEventTypes } from "../../GameEvents"
import { Step } from "../../Steps"
import { TriggerEvaluation } from "../TriggerEvaluation"

describe("TriggerEvaluation Service", () => {
  describe("collectPermanentsFromBattlefield", () => {
    test("returns empty array when no permanents on battlefield", () => {
      const { game } = createStartedGame()

      const permanents =
        TriggerEvaluation.collectPermanentsFromBattlefield(game)

      expect(permanents).toEqual([])
    })

    test("collects permanents from single player", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      const permanents =
        TriggerEvaluation.collectPermanentsFromBattlefield(game)

      expect(permanents).toHaveLength(1)
      expect(permanents[0]).toEqual({
        permanent: creature,
        controllerId: player1.id,
      })
    })

    test("collects permanents from both players", () => {
      const { game, player1, player2 } = createStartedGame()
      const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
      const creature2 = createTestCreature(player2.id, "creature-2", 3, 3)
      addCreatureToBattlefield(game, player1.id, creature1)
      addCreatureToBattlefield(game, player2.id, creature2)

      const permanents =
        TriggerEvaluation.collectPermanentsFromBattlefield(game)

      expect(permanents).toHaveLength(2)
      expect(permanents).toContainEqual({
        permanent: creature1,
        controllerId: player1.id,
      })
      expect(permanents).toContainEqual({
        permanent: creature2,
        controllerId: player2.id,
      })
    })

    test("collects multiple permanents from same player", () => {
      const { game, player1 } = createStartedGame()
      const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
      const creature2 = createTestCreature(player1.id, "creature-2", 3, 3)
      addCreatureToBattlefield(game, player1.id, creature1)
      addCreatureToBattlefield(game, player1.id, creature2)

      const permanents =
        TriggerEvaluation.collectPermanentsFromBattlefield(game)

      expect(permanents).toHaveLength(2)
    })
  })

  describe("findMatchingTriggers", () => {
    test("returns empty array when no permanents have triggers", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toEqual([])
    })

    test("returns empty array when event type doesn't match", () => {
      const { game, player1 } = createStartedGame()
      const mockEffect = vi.fn()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)

      // Add a trigger that watches for ZONE_CHANGED
      creature.definition.triggers = [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: () => true,
          effect: mockEffect,
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature)

      // Fire a different event type
      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toEqual([])
    })

    test("returns trigger when event type matches and condition is true", () => {
      const { game, player1 } = createStartedGame()
      const mockEffect = vi.fn()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)

      creature.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => true,
          effect: mockEffect,
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature)

      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toHaveLength(1)
      expect(triggers[0]).toEqual({
        effect: mockEffect,
        controllerId: player1.id,
        source: creature,
      })
    })

    test("does not return trigger when condition is false", () => {
      const { game, player1 } = createStartedGame()
      const mockEffect = vi.fn()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)

      creature.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => false, // Condition is false
          effect: mockEffect,
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature)

      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toEqual([])
    })

    test("collects multiple matching triggers from different permanents", () => {
      const { game, player1, player2 } = createStartedGame()
      const mockEffect1 = vi.fn()
      const mockEffect2 = vi.fn()

      const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
      creature1.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => true,
          effect: mockEffect1,
        },
      ]

      const creature2 = createTestCreature(player2.id, "creature-2", 3, 3)
      creature2.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => true,
          effect: mockEffect2,
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature1)
      addCreatureToBattlefield(game, player2.id, creature2)

      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toHaveLength(2)
    })

    test("collects multiple triggers from same permanent", () => {
      const { game, player1 } = createStartedGame()
      const mockEffect1 = vi.fn()
      const mockEffect2 = vi.fn()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      creature.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => true,
          effect: mockEffect1,
        },
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: () => true,
          effect: mockEffect2,
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature)

      const triggers = TriggerEvaluation.findMatchingTriggers(game, {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      })

      expect(triggers).toHaveLength(2)
    })

    test("condition receives game, event, and permanent", () => {
      const { game, player1 } = createStartedGame()
      const mockCondition = vi.fn().mockReturnValue(true)
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)

      creature.definition.triggers = [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: mockCondition,
          effect: vi.fn(),
        },
      ]

      addCreatureToBattlefield(game, player1.id, creature)

      const event = {
        type: GameEventTypes.STEP_STARTED,
        step: Step.FIRST_MAIN,
        activePlayerId: player1.id,
      }

      TriggerEvaluation.findMatchingTriggers(game, event)

      expect(mockCondition).toHaveBeenCalledWith(game, event, creature)
    })
  })
})
