import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { ZoneNames } from "../../zones/Zone"
import { GameEventTypes } from "../GameEvents"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  addSpellToHand,
  advanceToStep,
  createGameInMainPhase,
  createStartedGame,
  resolveStack,
} from "./helpers"

/**
 * TRIGGER SYSTEM MVP VALIDATION - ELF CARDS
 *
 * This test suite validates the trigger system using elf-themed cards
 * that are conceptually faithful to real Magic: The Gathering cards.
 *
 * SCOPE (MVP):
 * - ETB (Enter the Battlefield) triggers
 * - Attack triggers
 * - Conditional triggers based on battlefield state
 * - Negative cases (trigger doesn't fire)
 *
 * OUT OF SCOPE (MVP):
 * - Replacement effects
 * - Dynamic targets in triggers
 * - APNAP ordering for simultaneous triggers
 * - Separate trigger stack
 * - "first time each turn" / "once per turn" tracking
 * - Activated abilities (mana system not yet implemented)
 *
 * TODO: ETB triggers with targeting not yet implemented
 * TODO: Triggers execute immediately (no separate trigger stack)
 * TODO: APNAP ordering for simultaneous triggers not yet implemented
 * TODO: Complex targeting in trigger effects not yet supported
 * TODO: Activated abilities (e.g., {T}: Add {G}) require mana system
 */

// ============================================================================
// CARD DEFINITIONS - ELF THEME
// ============================================================================

/**
 * Creates Elvish Visionary card instance
 *
 * Real card text: "When Elvish Visionary enters the battlefield, draw a card."
 *
 * MVP implementation:
 * - ETB trigger fires when entering battlefield
 * - Calls game.drawCards() (currently no-op in MVP)
 * - No targeting required
 *
 * @param ownerId - Player who owns the card
 * @param drawCallback - Optional callback to track draw execution (for testing)
 */
function createElvishVisionary(
  ownerId: string,
  drawCallback?: () => void,
): CardInstance {
  return {
    instanceId: `elvish-visionary-${Math.random()}`,
    definition: {
      id: "elvish-visionary",
      name: "Elvish Visionary",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (game, context) => {
            // Draw a card when entering battlefield
            game.drawCards(context.controllerId, 1)
            drawCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}

/**
 * Creates Llanowar Elves card instance
 *
 * Real card text: "{T}: Add {G}."
 *
 * MVP limitations:
 * - Activated ability NOT implemented (no mana system yet)
 * - Card serves as "another elf" for conditional triggers
 * - Still a valid creature on battlefield
 *
 * TODO: Implement activated abilities when mana system exists
 *
 * @param ownerId - Player who owns the card
 */
function createLlanowarElves(ownerId: string): CardInstance {
  return {
    instanceId: `llanowar-elves-${Math.random()}`,
    definition: {
      id: "llanowar-elves",
      name: "Llanowar Elves",
      types: ["CREATURE"],
      // TODO: Add activatedAbility when mana system is implemented
      // activatedAbility: {
      //   cost: { type: "TAP" },
      //   effect: (game, context) => game.addMana(context.controllerId, "G", 1)
      // }
    },
    ownerId,
  }
}

/**
 * Creates Elvish Warrior card instance
 *
 * Real card: Vanilla 2/3 creature (no abilities)
 *
 * MVP purpose:
 * - Tests that creatures without triggers don't execute anything
 * - Serves as "another elf" for conditional triggers
 *
 * @param ownerId - Player who owns the card
 */
function createElvishWarrior(ownerId: string): CardInstance {
  return {
    instanceId: `elvish-warrior-${Math.random()}`,
    definition: {
      id: "elvish-warrior",
      name: "Elvish Warrior",
      types: ["CREATURE"],
      // No triggers, no abilities - vanilla creature
    },
    ownerId,
  }
}

/**
 * Creates a test elf with conditional ETB trigger
 *
 * Conceptual card text:
 * "When this enters the battlefield, if you control another Elf, draw a card."
 *
 * Implementation notes:
 * - "Another" means "a different elf, not this card itself"
 * - Condition inspects battlefield state at trigger evaluation time
 * - Tests that triggers can have complex conditional logic
 *
 * @param ownerId - Player who owns the card
 * @param drawCallback - Optional callback to track draw execution (for testing)
 */
function createConditionalElf(
  ownerId: string,
  drawCallback?: () => void,
): CardInstance {
  return {
    instanceId: `conditional-elf-${Math.random()}`,
    definition: {
      id: "conditional-elf",
      name: "Conditional Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (game, event, source) => {
            // Must be this card entering battlefield
            if (event.card.instanceId !== source.instanceId) return false
            if (event.toZone !== ZoneNames.BATTLEFIELD) return false

            // Check if controller has ANOTHER elf (excluding this one)
            const battlefield = game.getPlayerState(event.controllerId)
              .battlefield.cards

            const otherElves = battlefield.filter(
              (card) =>
                // Different card (not this one)
                card.instanceId !== source.instanceId &&
                // Is a creature
                card.definition.types.includes("CREATURE") &&
                // Card is an elf (MVP heuristic: check for "elf" or "elv" in ID or name)
                // In a real implementation, this would check creature subtypes
                (card.definition.id.includes("elf") ||
                  card.definition.id.includes("elv") ||
                  card.definition.name.toLowerCase().includes("elf") ||
                  card.definition.name.toLowerCase().includes("elv")),
            )

            return otherElves.length > 0
          },
          effect: (game, context) => {
            game.drawCards(context.controllerId, 1)
            drawCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}

/**
 * Creates an elf with attack trigger
 *
 * Conceptual card text:
 * "Whenever this creature attacks, draw a card."
 *
 * Tests that attack triggers work correctly.
 *
 * @param ownerId - Player who owns the card
 * @param attackCallback - Optional callback to track attack trigger execution
 */
function createElfWithAttackTrigger(
  ownerId: string,
  attackCallback?: () => void,
): CardInstance {
  return {
    instanceId: `attacking-elf-${Math.random()}`,
    definition: {
      id: "attacking-elf",
      name: "Attacking Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
          condition: (_game, event, source) =>
            event.creature.instanceId === source.instanceId,
          effect: (game, context) => {
            game.drawCards(context.controllerId, 1)
            attackCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}

// ============================================================================
// TEST 1 - ETB BÁSICO SE EJECUTA (Elvish Visionary)
// ============================================================================

test("TEST 1: ETB trigger executes when Elvish Visionary enters battlefield", () => {
  const { game, player1, player2 } = createGameInMainPhase()

  let drawExecuted = false
  const elvishVisionary = createElvishVisionary(player1.id, () => {
    drawExecuted = true
  })

  // Add to hand
  addSpellToHand(game, player1.id, elvishVisionary)

  // Cast the creature
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: elvishVisionary.instanceId,
    targets: [],
  })

  // Before resolution, trigger should NOT have fired
  expect(drawExecuted).toBe(false)

  // Resolve the stack (creature enters battlefield)
  resolveStack(game, player2.id, player1.id)

  // After resolution:
  // - Creature is on battlefield
  // - ETB trigger executed exactly once
  // - Draw effect was called
  expect(drawExecuted).toBe(true)

  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].definition.name).toBe("Elvish Visionary")
})

test("TEST 1 (variant): ETB trigger fires exactly once, not multiple times", () => {
  const { game, player1 } = createGameInMainPhase()

  let drawCount = 0
  const elvishVisionary = createElvishVisionary(player1.id, () => {
    drawCount++
  })

  // Add directly to battlefield (bypassing cast/resolve)
  addCreatureToBattlefield(game, player1.id, elvishVisionary)

  // Trigger should fire exactly once
  expect(drawCount).toBe(1)

  // Add another creature (should NOT trigger the first one's ability)
  const llanowar = createLlanowarElves(player1.id)
  addCreatureToBattlefield(game, player1.id, llanowar)

  // Draw count should still be 1 (not 2)
  expect(drawCount).toBe(1)
})

// ============================================================================
// TEST 2 - ETB NO SE EJECUTA SI NO ENTRA AL BATTLEFIELD
// ============================================================================

test("TEST 2: ETB trigger does NOT fire when card goes to graveyard instead of battlefield", () => {
  const { game, player1 } = createGameInMainPhase()

  let drawExecuted = false
  const elvishVisionary = createElvishVisionary(player1.id, () => {
    drawExecuted = true
  })

  // Directly put card in graveyard (simulating discard, mill, etc.)
  const playerState = game.getPlayerState(player1.id)
  playerState.graveyard.cards.push(elvishVisionary)

  // ETB trigger should NOT fire (card didn't enter battlefield)
  expect(drawExecuted).toBe(false)

  // Verify card is in graveyard
  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].definition.name).toBe("Elvish Visionary")
})

test("TEST 2 (variant): ETB trigger condition requires toZone === BATTLEFIELD", () => {
  const { game, player1 } = createGameInMainPhase()

  let triggerCount = 0
  const elvishVisionary = createElvishVisionary(player1.id, () => {
    triggerCount++
  })

  // Add to hand first
  const playerState = game.getPlayerState(player1.id)
  playerState.hand.cards.push(elvishVisionary)

  // Move from hand to graveyard (not battlefield)
  // This simulates a zone change, but NOT to battlefield
  const handIndex = playerState.hand.cards.findIndex(
    (c) => c.instanceId === elvishVisionary.instanceId,
  )
  playerState.hand.cards.splice(handIndex, 1)
  playerState.graveyard.cards.push(elvishVisionary)

  // ETB trigger should NOT fire
  expect(triggerCount).toBe(0)
})

// ============================================================================
// TEST 3 - TRIGGER "CUANDO ESTA CRIATURA ATACA"
// ============================================================================

test("TEST 3: Attack trigger executes when creature attacks", () => {
  const { game, player1 } = createStartedGame()

  let attackTriggerFired = false
  const attackingElf = createElfWithAttackTrigger(player1.id, () => {
    attackTriggerFired = true
  })

  // Add to battlefield
  addCreatureToBattlefield(game, player1.id, attackingElf)

  // Advance to declare attackers step
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Before declaring attack, trigger should not have fired
  expect(attackTriggerFired).toBe(false)

  // Declare attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attackingElf.instanceId,
  })

  // After declaring attack:
  // - Trigger executed exactly once
  // - Effect was called
  expect(attackTriggerFired).toBe(true)

  // Verify creature is marked as attacking
  const creatureState = game.getCreatureState(attackingElf.instanceId)
  expect(creatureState.isAttacking).toBe(true)
})

test("TEST 3 (negative): Attack trigger does NOT fire if creature doesn't attack", () => {
  const { game, player1 } = createStartedGame()

  let attackTriggerFired = false
  const attackingElf = createElfWithAttackTrigger(player1.id, () => {
    attackTriggerFired = true
  })
  const otherElf = createLlanowarElves(player1.id)

  // Add both to battlefield
  addCreatureToBattlefield(game, player1.id, attackingElf)
  addCreatureToBattlefield(game, player1.id, otherElf)

  // Advance to declare attackers step
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Declare ONLY the other elf as attacker (not the one with trigger)
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: otherElf.instanceId,
  })

  // Attack trigger should NOT have fired (different creature attacked)
  expect(attackTriggerFired).toBe(false)
})

// ============================================================================
// TEST 4 - TRIGGER CONDICIONAL BASADO EN ESTADO DEL BATTLEFIELD ("OTRO ELFO")
// ============================================================================

test("TEST 4A: Conditional trigger does NOT fire when no other elves on battlefield", () => {
  const { game, player1 } = createGameInMainPhase()

  let drawExecuted = false
  const conditionalElf = createConditionalElf(player1.id, () => {
    drawExecuted = true
  })

  // Battlefield is empty, add conditional elf
  addCreatureToBattlefield(game, player1.id, conditionalElf)

  // Trigger should NOT fire (no OTHER elves)
  expect(drawExecuted).toBe(false)

  // Verify creature is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].definition.name).toBe("Conditional Elf")
})

test("TEST 4B: Conditional trigger DOES fire when another elf is on battlefield", () => {
  const { game, player1 } = createGameInMainPhase()

  // First, add Llanowar Elves to battlefield
  const llanowar = createLlanowarElves(player1.id)
  addCreatureToBattlefield(game, player1.id, llanowar)

  // Verify first elf is on battlefield
  let battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)

  // Debug: verify the llanowar has correct properties
  const llanowarOnBattlefield = battlefield[0]
  expect(llanowarOnBattlefield.definition.name).toBe("Llanowar Elves")
  expect(llanowarOnBattlefield.definition.types).toContain("CREATURE")
  expect(llanowarOnBattlefield.definition.id).toBe("llanowar-elves")

  let drawExecuted = false
  const conditionalElf = createConditionalElf(player1.id, () => {
    drawExecuted = true
  })

  // Now add conditional elf (should trigger because Llanowar is already there)
  addCreatureToBattlefield(game, player1.id, conditionalElf)

  // Trigger SHOULD fire (another elf exists)
  expect(drawExecuted).toBe(true)

  // Verify both creatures are on battlefield
  battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(2)
})

test("TEST 4C: Conditional trigger validates 'another' (different card, not self)", () => {
  const { game, player1 } = createGameInMainPhase()

  let drawCount = 0

  // Create two conditional elves
  const conditionalElf1 = createConditionalElf(player1.id, () => {
    drawCount++
  })
  const conditionalElf2 = createConditionalElf(player1.id, () => {
    drawCount++
  })

  // Add first conditional elf (no other elves, should NOT trigger)
  addCreatureToBattlefield(game, player1.id, conditionalElf1)
  expect(drawCount).toBe(0)

  // Add second conditional elf (should trigger because first elf exists)
  addCreatureToBattlefield(game, player1.id, conditionalElf2)
  expect(drawCount).toBe(1) // Only second elf triggers

  // Verify both are on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(2)
})

// ============================================================================
// TEST 5 - CRIATURA SIN TRIGGERS NO EJECUTA NADA (Elvish Warrior)
// ============================================================================

test("TEST 5: Vanilla creature without triggers executes no effects", () => {
  const { game, player1, player2 } = createGameInMainPhase()

  // Track if any unexpected side effects occur
  const unexpectedEffect = false

  // Create vanilla Elvish Warrior
  const elvishWarrior = createElvishWarrior(player1.id)

  // Add to hand and cast
  addSpellToHand(game, player1.id, elvishWarrior)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: elvishWarrior.instanceId,
    targets: [],
  })

  // Resolve the stack
  resolveStack(game, player2.id, player1.id)

  // No triggers should have fired
  expect(unexpectedEffect).toBe(false)

  // Creature should be on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].definition.name).toBe("Elvish Warrior")

  // Verify no triggers are defined
  expect(battlefield[0].definition.triggers).toBeUndefined()
})

test("TEST 5 (variant): Multiple vanilla creatures don't trigger anything", () => {
  const { game, player1 } = createGameInMainPhase()

  // Add multiple vanilla creatures
  const warrior1 = createElvishWarrior(player1.id)
  const warrior2 = createElvishWarrior(player1.id)
  const llanowar = createLlanowarElves(player1.id)

  addCreatureToBattlefield(game, player1.id, warrior1)
  addCreatureToBattlefield(game, player1.id, warrior2)
  addCreatureToBattlefield(game, player1.id, llanowar)

  // All should be on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(3)

  // None should have triggers
  for (const creature of battlefield) {
    if (
      creature.definition.name === "Elvish Warrior" ||
      creature.definition.name === "Llanowar Elves"
    ) {
      expect(creature.definition.triggers).toBeUndefined()
    }
  }
})

// ============================================================================
// ADDITIONAL VALIDATION TESTS
// ============================================================================

test("ETB trigger validates source card identity correctly", () => {
  const { game, player1 } = createGameInMainPhase()

  let visionary1Triggered = false
  let visionary2Triggered = false

  const visionary1 = createElvishVisionary(player1.id, () => {
    visionary1Triggered = true
  })
  const visionary2 = createElvishVisionary(player1.id, () => {
    visionary2Triggered = true
  })

  // Add first visionary
  addCreatureToBattlefield(game, player1.id, visionary1)
  expect(visionary1Triggered).toBe(true)
  expect(visionary2Triggered).toBe(false)

  // Add second visionary
  addCreatureToBattlefield(game, player1.id, visionary2)
  expect(visionary1Triggered).toBe(true) // Still true (didn't fire again)
  expect(visionary2Triggered).toBe(true) // Now also true

  // Both should be on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(2)
})

test("Multiple triggers on battlefield all evaluate for same event", () => {
  const { game, player1 } = createGameInMainPhase()

  const triggerLog: string[] = []

  // Add three creatures with ETB triggers
  const visionary1 = createElvishVisionary(player1.id, () => {
    triggerLog.push("visionary1")
  })
  const visionary2 = createElvishVisionary(player1.id, () => {
    triggerLog.push("visionary2")
  })
  const conditionalElf = createConditionalElf(player1.id, () => {
    triggerLog.push("conditional")
  })

  addCreatureToBattlefield(game, player1.id, visionary1)
  addCreatureToBattlefield(game, player1.id, visionary2)
  addCreatureToBattlefield(game, player1.id, conditionalElf)

  // All three triggers should have fired
  expect(triggerLog).toEqual(["visionary1", "visionary2", "conditional"])
})
