import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { EffectContext } from "../../effects/EffectContext"
import type { Game } from "../Game"
import { GameEventTypes } from "../GameEvents"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  addSpellToHand,
  advanceToStep,
  createCreatureWithETBTrigger,
  createGameInMainPhase,
  createStartedGame,
  createTestCreature,
  resolveStack,
} from "./helpers"

/**
 * Tests for the trigger system (triggered abilities)
 *
 * The trigger system allows cards to declare "when X happens, do Y" rules
 * without actively subscribing to events or maintaining internal state.
 *
 * Key principles:
 * - Cards declare triggers (conditions + effects)
 * - Game evaluates triggers when events occur
 * - No event bus, no subscriptions, no active listeners
 */

// ============================================================================
// ETB TRIGGERS (Zone Change to Battlefield)
// ============================================================================

test("trigger fires when card enters the battlefield", () => {
  const { game, player1, player2 } = createGameInMainPhase()

  let triggerExecuted = false

  const creatureWithETBTrigger: CardInstance = {
    instanceId: "elvish-visionary",
    definition: {
      id: "elvish-visionary",
      name: "Elvish Visionary",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) => {
            return (
              event.card.instanceId === source.instanceId &&
              event.toZone === "BATTLEFIELD"
            )
          },
          effect: (_game: Game, _context: EffectContext) => {
            triggerExecuted = true
            // In a real implementation: game.drawCards(context.controllerId, 1)
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, creatureWithETBTrigger)

  // Cast the creature
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureWithETBTrigger.instanceId,
    targets: [],
  })

  // Before resolution, trigger should not have fired
  expect(triggerExecuted).toBe(false)

  // Resolve the stack
  resolveStack(game, player2.id, player1.id)

  // After resolution, trigger should have fired
  expect(triggerExecuted).toBe(true)

  // Verify creature is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].instanceId).toBe("elvish-visionary")
})

test("trigger does NOT fire for other cards entering battlefield", () => {
  const { game, player1 } = createGameInMainPhase()

  let triggerExecutionCount = 0

  const creatureWithETBTrigger: CardInstance = {
    instanceId: "elf-1",
    definition: {
      id: "elf",
      name: "Llanowar Elves",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) => {
            // Only trigger when THIS card enters
            return (
              event.card.instanceId === source.instanceId &&
              event.toZone === "BATTLEFIELD"
            )
          },
          effect: () => {
            triggerExecutionCount++
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  const creatureWithoutTrigger: CardInstance = {
    instanceId: "elf-2",
    definition: {
      id: "elf",
      name: "Llanowar Elves",
      types: ["CREATURE"],
    },
    ownerId: player1.id,
  }

  // Add first creature to battlefield
  addCreatureToBattlefield(game, player1.id, creatureWithETBTrigger)
  expect(triggerExecutionCount).toBe(1)

  // Add second creature to battlefield (should NOT trigger the first one's ability)
  addCreatureToBattlefield(game, player1.id, creatureWithoutTrigger)
  expect(triggerExecutionCount).toBe(1) // Still 1, not 2

  // Verify both creatures are on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(2)
})

test("multiple permanents with ETB triggers all fire", () => {
  const { game, player1 } = createGameInMainPhase()

  const executionLog: string[] = []

  // Add three elves to battlefield, each with its own ETB trigger
  addCreatureToBattlefield(
    game,
    player1.id,
    createCreatureWithETBTrigger("elf-1", player1.id, () =>
      executionLog.push("elf-1"),
    ),
  )
  addCreatureToBattlefield(
    game,
    player1.id,
    createCreatureWithETBTrigger("elf-2", player1.id, () =>
      executionLog.push("elf-2"),
    ),
  )
  addCreatureToBattlefield(
    game,
    player1.id,
    createCreatureWithETBTrigger("elf-3", player1.id, () =>
      executionLog.push("elf-3"),
    ),
  )

  // Each trigger should have fired exactly once
  expect(executionLog).toEqual(["elf-1", "elf-2", "elf-3"])
})

test("trigger condition can inspect game state", () => {
  const { game, player1 } = createGameInMainPhase()

  let triggerExecuted = false

  const conditionalTrigger: CardInstance = {
    instanceId: "conditional-elf",
    definition: {
      id: "conditional-elf",
      name: "Conditional Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (game, event, source) => {
            if (event.card.instanceId !== source.instanceId) return false
            if (event.toZone !== "BATTLEFIELD") return false

            // Only trigger if controller has at least 2 creatures on battlefield
            const battlefield = game.getPlayerState(event.controllerId)
              .battlefield.cards
            const creatureCount = battlefield.filter((card) =>
              card.definition.types.includes("CREATURE"),
            ).length

            return creatureCount >= 2
          },
          effect: () => {
            triggerExecuted = true
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  // Add first creature (should NOT trigger - only 1 creature total)
  addCreatureToBattlefield(game, player1.id, conditionalTrigger)
  expect(triggerExecuted).toBe(false)

  // Reset and try again with existing creature
  triggerExecuted = false
  const battlefield = game.getPlayerState(player1.id).battlefield
  battlefield.cards = [] // Clear battlefield

  // Add a different creature first
  addCreatureToBattlefield(game, player1.id, createTestCreature(player1.id))

  // Now add the conditional one (should trigger - 2 creatures total)
  addCreatureToBattlefield(game, player1.id, conditionalTrigger)
  expect(triggerExecuted).toBe(true)
})

// ============================================================================
// ATTACK TRIGGERS
// ============================================================================

test("trigger fires when creature attacks", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  let attackTriggerCount = 0

  const attackerWithTrigger: CardInstance = {
    instanceId: "attacker-1",
    definition: {
      id: "attacker",
      name: "Elf Warrior",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
          condition: (_game, event, source) => {
            // Trigger when THIS creature attacks
            return event.creature.instanceId === source.instanceId
          },
          effect: () => {
            attackTriggerCount++
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, attackerWithTrigger)

  // Declare attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attackerWithTrigger.instanceId,
  })

  // Trigger should have fired
  expect(attackTriggerCount).toBe(1)
})

test("attack trigger can observe ANY creature attacking", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  const attackersSeen: string[] = []

  const observerCreature: CardInstance = {
    instanceId: "observer",
    definition: {
      id: "observer",
      name: "Battle Observer",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
          condition: () => true, // Trigger for ANY creature attacking
          effect: (_game, _context) => {
            // In a real implementation, we'd extract the attacking creature from context
            attackersSeen.push("attack-observed")
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  const attacker1 = createTestCreature(player1.id, "attacker-1")
  const attacker2 = createTestCreature(player1.id, "attacker-2")

  addCreatureToBattlefield(game, player1.id, observerCreature)
  addCreatureToBattlefield(game, player1.id, attacker1)
  addCreatureToBattlefield(game, player1.id, attacker2)

  // Declare first attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker1.instanceId,
  })

  expect(attackersSeen).toHaveLength(1)

  // Declare second attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker2.instanceId,
  })

  expect(attackersSeen).toHaveLength(2)
})

// ============================================================================
// STEP TRIGGERS
// ============================================================================

test("trigger fires at beginning of step", () => {
  const { game, player1 } = createStartedGame()

  let combatTriggerCount = 0

  const combatTriggerCreature: CardInstance = {
    instanceId: "combat-elf",
    definition: {
      id: "combat-elf",
      name: "Combat Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: (_game, event, _source) => {
            return event.step === Step.BEGINNING_OF_COMBAT
          },
          effect: () => {
            combatTriggerCount++
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, combatTriggerCreature)

  // Advance to combat
  advanceToStep(game, Step.BEGINNING_OF_COMBAT)

  // Trigger should have fired
  expect(combatTriggerCount).toBe(1)

  // Advance to next turn
  game.apply({ type: "END_TURN", playerId: player1.id })
  advanceToStep(game, Step.BEGINNING_OF_COMBAT)

  // Trigger should fire again
  expect(combatTriggerCount).toBe(2)
})

test("trigger fires at beginning of upkeep", () => {
  const { game, player1 } = createStartedGame()

  let upkeepTriggerCount = 0

  const upkeepTriggerCreature: CardInstance = {
    instanceId: "upkeep-elf",
    definition: {
      id: "upkeep-elf",
      name: "Upkeep Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.STEP_STARTED,
          condition: (_game, event, _source) => {
            return event.step === Step.UPKEEP
          },
          effect: () => {
            upkeepTriggerCount++
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  // Start at UNTAP
  expect(game.currentStep).toBe(Step.UNTAP)

  addCreatureToBattlefield(game, player1.id, upkeepTriggerCreature)

  // Advance to UPKEEP
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  expect(upkeepTriggerCount).toBe(1)
})

// ============================================================================
// EDGE CASES
// ============================================================================

test("trigger does not fire if condition is false", () => {
  const { game, player1 } = createGameInMainPhase()

  let triggerExecuted = false

  const creatureWithFalseCondition: CardInstance = {
    instanceId: "never-triggers",
    definition: {
      id: "never-triggers",
      name: "Never Triggers",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: () => false, // Never triggers
          effect: () => {
            triggerExecuted = true
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, creatureWithFalseCondition)

  expect(triggerExecuted).toBe(false)
})

test("card with multiple triggers - all fire when conditions met", () => {
  const { game, player1 } = createGameInMainPhase()

  const executionLog: string[] = []

  const multiTriggerCreature: CardInstance = {
    instanceId: "multi-trigger",
    definition: {
      id: "multi-trigger",
      name: "Multi Trigger",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === "BATTLEFIELD",
          effect: () => {
            executionLog.push("ETB-1")
          },
        },
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === "BATTLEFIELD",
          effect: () => {
            executionLog.push("ETB-2")
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, multiTriggerCreature)

  expect(executionLog).toEqual(["ETB-1", "ETB-2"])
})

test("trigger fires even if source permanent leaves battlefield before resolution", () => {
  // This is a simplified test for the future when triggers go on the stack
  // For MVP, triggers execute immediately so this scenario doesn't apply yet
  // TODO: Update this test when triggers are added to a separate stack
  const { game, player1 } = createGameInMainPhase()

  let triggerExecuted = false

  const creature: CardInstance = {
    instanceId: "leaves-immediately",
    definition: {
      id: "leaves",
      name: "Leaves",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === "BATTLEFIELD",
          effect: () => {
            triggerExecuted = true
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, creature)

  // MVP: Trigger executes immediately, so it always executes
  expect(triggerExecuted).toBe(true)

  // In the future with stacked triggers:
  // - Trigger would go on stack
  // - Source could leave battlefield
  // - Trigger would still resolve using Last Known Information
})
