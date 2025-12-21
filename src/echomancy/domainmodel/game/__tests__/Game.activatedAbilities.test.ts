import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { NoOpEffect } from "../../effects/impl/NoOpEffect"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createTestCreature,
} from "./helpers"

// ============================================================================
// Helper: Create a creature with an activated ability
// ============================================================================

function createCreatureWithActivatedAbility(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `creature-with-ability-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "creature-with-ability-def",
      name: "Creature with Ability",
      types: ["CREATURE"],
      activatedAbility: {
        cost: { type: "TAP" },
        effect: new NoOpEffect(), // No-op effect; this test only checks activation/stacking
      },
    },
    ownerId,
  }
}

// ============================================================================
// TEST 1 — A creature with activated ability can activate it
// ============================================================================

test("a creature with activated ability can activate it when player has priority", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  // Player 1 has priority in main phase, creature is untapped
  expect(() => {
    game.apply({
      type: "ACTIVATE_ABILITY",
      playerId: player1.id,
      permanentId: creature.instanceId,
    })
  }).not.toThrow()

  // Ability should be on stack
  const stack = game.getStack()
  expect(stack).toHaveLength(1)
})

// ============================================================================
// TEST 2 — Activating an ability pays its cost (tap)
// ============================================================================

test("activating an ability with {T} cost taps the creature", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  // Verify creature starts untapped
  const stateBefore = game.getCreatureState(creature.instanceId)
  expect(stateBefore.isTapped).toBe(false)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Creature should be tapped
  const stateAfter = game.getCreatureState(creature.instanceId)
  expect(stateAfter.isTapped).toBe(true)

  // Ability should be on stack
  const stack = game.getStack()
  expect(stack).toHaveLength(1)
})

// ============================================================================
// TEST 3 — A tapped creature cannot activate abilities with {T}
// ============================================================================

test("a tapped creature cannot activate abilities with {T} cost", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  // Tap the creature
  game.tapPermanent(creature.instanceId)

  // Verify creature is tapped
  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isTapped).toBe(true)

  // Attempting to activate should fail
  expect(() => {
    game.apply({
      type: "ACTIVATE_ABILITY",
      playerId: player1.id,
      permanentId: creature.instanceId,
    })
  }).toThrow()

  // Stack should remain empty
  const stack = game.getStack()
  expect(stack).toHaveLength(0)
})

// ============================================================================
// TEST 4 — Activating an ability consumes priority
// ============================================================================

test("activating an ability gives priority to opponent", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Player 2 should now have priority
  const allowedActionsPlayer1 = game.getAllowedActionsFor(player1.id)
  const allowedActionsPlayer2 = game.getAllowedActionsFor(player2.id)

  expect(allowedActionsPlayer1).toHaveLength(0)
  expect(allowedActionsPlayer2.length).toBeGreaterThan(0)
})

// ============================================================================
// TEST 5 — The ability resolves using the stack
// ============================================================================

test("the ability resolves when both players pass priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Ability on stack
  expect(game.getStack()).toHaveLength(1)

  // Both players pass priority
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Ability should have resolved and left the stack
  expect(game.getStack()).toHaveLength(0)
})

// ============================================================================
// TEST 6 — The effect executes exactly once
// ============================================================================

test("the ability effect executes exactly once when resolved", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let executionCount = 0
  const countingEffect = {
    resolve: () => {
      executionCount++
    },
  }

  const creature: CardInstance = {
    instanceId: "counting-creature",
    definition: {
      id: "counting-creature-def",
      name: "Counting Creature",
      types: ["CREATURE"],
      activatedAbility: {
        cost: { type: "TAP" },
        effect: countingEffect,
      },
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, creature)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Resolve
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  expect(executionCount).toBe(1)
})

// ============================================================================
// TEST 7 — Resolving an ability does NOT trigger ETB or LTB
// ============================================================================

test("resolving an ability does not trigger ETB or LTB effects", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbCount = 0
  const etbEffect = {
    resolve: () => {
      etbCount++
    },
  }

  const creature: CardInstance = {
    instanceId: "etb-creature",
    definition: {
      id: "etb-creature-def",
      name: "ETB Creature",
      types: ["CREATURE"],
      onEnterBattlefield: etbEffect,
      activatedAbility: {
        cost: { type: "TAP" },
        effect: new NoOpEffect(),
      },
    },
    ownerId: player1.id,
  }

  // Add to battlefield (ETB should fire once)
  addCreatureToBattlefield(game, player1.id, creature)
  const playerState = game.getPlayerState(player1.id)
  // Manually trigger ETB since we added directly to battlefield
  if (creature.definition.onEnterBattlefield) {
    creature.definition.onEnterBattlefield.resolve(game, {
      source: creature,
      controllerId: player1.id,
      targets: [],
    })
  }

  expect(etbCount).toBe(1)

  // Activate and resolve ability
  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // ETB should still be 1 (not triggered again)
  expect(etbCount).toBe(1)

  // Creature should still be on battlefield
  expect(playerState.battlefield.cards).toHaveLength(1)
})

// ============================================================================
// TEST 8 — After resolution, priority returns to active player
// ============================================================================

test("after ability resolves, priority returns to active player", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Resolve
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Player 1 (active player) should have priority
  const allowedActionsPlayer1 = game.getAllowedActionsFor(player1.id)
  const allowedActionsPlayer2 = game.getAllowedActionsFor(player2.id)

  expect(allowedActionsPlayer1.length).toBeGreaterThan(0)
  expect(allowedActionsPlayer2).toHaveLength(0)
})

// ============================================================================
// TEST 9 — Activating an ability does NOT count as casting a spell
// ============================================================================

test("activating an ability does not count as casting a spell", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  const stackBefore = game.getStack()
  expect(stackBefore).toHaveLength(0)

  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  const stackAfter = game.getStack()
  expect(stackAfter).toHaveLength(1)

  // The stack item should NOT be a SpellOnStack - it should be an AbilityOnStack
  // This is verified by the type system and the implementation
  // A spell comes from hand and moves to graveyard/battlefield
  // An ability comes from battlefield and doesn't move the permanent

  // Resolve the ability
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Creature should still be on battlefield (not moved to graveyard like a spell)
  const playerState = game.getPlayerState(player1.id)
  expect(playerState.battlefield.cards).toHaveLength(1)
  expect(playerState.graveyard.cards).toHaveLength(0)
})

// ============================================================================
// TEST 10 — Cannot activate ability without priority
// ============================================================================

test("cannot activate an ability without priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  // Give priority to player 2
  const dummyCreature = createTestCreature(player2.id)
  addCreatureToBattlefield(game, player2.id, dummyCreature)

  // Player 1 casts a spell to give priority to player 2
  const spell = createTestCreature(player1.id, "spell-instance")
  const playerState = game.getPlayerState(player1.id)
  playerState.hand.cards.push(spell)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  // Now player 2 has priority, player 1 doesn't
  const allowedActionsPlayer1 = game.getAllowedActionsFor(player1.id)
  expect(allowedActionsPlayer1).toHaveLength(0)

  // Player 1 tries to activate ability without priority
  expect(() => {
    game.apply({
      type: "ACTIVATE_ABILITY",
      playerId: player1.id,
      permanentId: creature.instanceId,
    })
  }).toThrow()

  // Stack should only have the spell (not the ability)
  const stack = game.getStack()
  expect(stack).toHaveLength(1)
})

// ============================================================================
// Additional test: Verify allowed actions include ACTIVATE_ABILITY
// ============================================================================

test("ACTIVATE_ABILITY appears in allowed actions when applicable", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createCreatureWithActivatedAbility(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  const allowedActions = game.getAllowedActionsFor(player1.id)
  expect(allowedActions).toContain("ACTIVATE_ABILITY")
})

test("ACTIVATE_ABILITY does not appear in allowed actions when no abilities available", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Regular creature without activated ability
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  const allowedActions = game.getAllowedActionsFor(player1.id)
  expect(allowedActions).not.toContain("ACTIVATE_ABILITY")
})

// ============================================================================
// Last Known Information: Ability resolves even if source is removed
// ============================================================================

test("ability resolves even if source permanent has left the battlefield", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const executionOrder: string[] = []

  // Create a creature with an observable effect
  const creature: CardInstance = {
    instanceId: "creature-with-ability",
    definition: {
      id: "creature-with-ability-def",
      name: "Creature with Ability",
      types: ["CREATURE"],
      activatedAbility: {
        cost: { type: "TAP" },
        effect: {
          resolve(_g, _context) {
            executionOrder.push("ABILITY_RESOLVED")
          },
        },
      },
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, creature)

  // Activate the ability (puts it on stack)
  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  // Verify ability is on stack
  let stack = game.getStack()
  expect(stack).toHaveLength(1)
  expect(stack[0].kind).toBe("ABILITY")

  // Simulate the source permanent being destroyed (remove from battlefield)
  // This happens BEFORE the ability resolves
  const playerState = game.getPlayerState(player1.id)
  const creatureIndex = playerState.battlefield.cards.findIndex(
    (c) => c.instanceId === creature.instanceId,
  )
  expect(creatureIndex).toBeGreaterThanOrEqual(0)
  playerState.battlefield.cards.splice(creatureIndex, 1)

  // Verify creature is gone from battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(
    battlefield.find((c) => c.instanceId === creature.instanceId),
  ).toBeUndefined()

  // Resolve the ability (both players pass priority)
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify the ability resolved despite source being removed (Last Known Information)
  expect(executionOrder).toEqual(["ABILITY_RESOLVED"])

  // Stack should be empty after resolution
  stack = game.getStack()
  expect(stack).toHaveLength(0)
})
