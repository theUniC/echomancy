import { expect, test } from "vitest"
import { StaticAbilities } from "../../cards/CardDefinition"
import type { CardInstance } from "../../cards/CardInstance"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  addCreatureToBattlefieldWithSummoningSickness,
  advanceToStep,
  createStartedGame,
  createStartedGameWithDecks,
  createTestCreature,
} from "./helpers"

/**
 * Tests for Summoning Sickness (B1-04)
 *
 * This test suite validates summoning sickness rules:
 * - Creatures cannot attack the turn they enter battlefield
 * - Creatures cannot use tap abilities the turn they enter battlefield
 * - Summoning sickness is cleared at UNTAP step
 * - Haste bypasses summoning sickness
 *
 * MVP Scope:
 * - Basic summoning sickness flag (boolean)
 * - Attack restriction enforcement
 * - Tap ability restriction enforcement
 * - Haste keyword bypass
 *
 * Explicitly excluded from MVP:
 * - Control change tracking (no control-change effects exist yet)
 * - Untap symbol (Q) costs (only T costs checked)
 * - Temporary haste effects ("gains haste until end of turn")
 * - Creatures becoming non-creatures mid-game
 */

// ============================================================================
// Helper: Create creature with Haste
// ============================================================================

function createTestCreatureWithHaste(
  ownerId: string,
  instanceId?: string,
  power?: number,
  toughness?: number,
): CardInstance {
  const id = instanceId || `haste-creature-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "haste-creature-def",
      name: "Haste Creature",
      types: ["CREATURE"],
      staticAbilities: [StaticAbilities.HASTE],
      power,
      toughness,
    },
    ownerId,
  }
}

// ============================================================================
// Summoning Sickness: Basic Flag Tests
// ============================================================================

test("creature entering battlefield has summoning sickness", () => {
  const { game, player1 } = createStartedGame()

  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.hasSummoningSickness).toBe(true)
})

test("creature with summoning sickness is exported in state", () => {
  const { game, player1 } = createStartedGame()

  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  const exportedState = game.exportState()
  const playerState = exportedState.players[player1.id]
  const exportedCreature = playerState?.zones.battlefield.cards.find(
    (c) => c.instanceId === creature.instanceId,
  )

  expect(exportedCreature).toBeDefined()
  expect(exportedCreature?.creatureState?.hasSummoningSickness).toBe(true)
})

// ============================================================================
// Summoning Sickness: Attack Restriction Tests
// ============================================================================

test("creature with summoning sickness cannot attack", () => {
  const { game, player1 } = createStartedGame()

  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Try to declare attacker - should throw CreatureHasSummoningSicknessError
  expect(() => {
    game.apply({
      type: "DECLARE_ATTACKER",
      playerId: player1.id,
      creatureId: creature.instanceId,
    })
  }).toThrow(/has summoning sickness/)
})

test("creature can attack after summoning sickness is cleared", () => {
  const { game, player1, player2 } = createStartedGameWithDecks()

  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  // End turn and start next turn (summoning sickness should clear at UNTAP)
  advanceToStep(game, Step.CLEANUP)

  // Advance through CLEANUP to enter player2's turn
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Now we're in player2's UNTAP, advance through their entire turn
  while (
    game.currentStep !== Step.CLEANUP ||
    game.currentPlayerId !== player2.id
  ) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  // Advance from player2's CLEANUP to player1's UNTAP
  game.apply({ type: "ADVANCE_STEP", playerId: player2.id })

  // We should be in player1's UNTAP step of their next turn
  expect(game.currentStep).toBe(Step.UNTAP)
  expect(game.currentPlayerId).toBe(player1.id)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Now creature should be able to attack (no error)
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isAttacking).toBe(true)
  expect(creatureState.hasSummoningSickness).toBe(false)
})

// ============================================================================
// Summoning Sickness: Cleared at UNTAP Step
// ============================================================================

test("summoning sickness cleared at start of controller's next UNTAP step", () => {
  const { game, player1, player2 } = createStartedGame()

  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  // Verify summoning sickness is set
  let creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.hasSummoningSickness).toBe(true)

  // End turn and start next turn
  advanceToStep(game, Step.CLEANUP)

  // Advance through CLEANUP to enter player2's turn
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Now we're in player2's UNTAP, advance through their entire turn
  while (
    game.currentStep !== Step.CLEANUP ||
    game.currentPlayerId !== player2.id
  ) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  // Advance from player2's CLEANUP to player1's UNTAP
  game.apply({ type: "ADVANCE_STEP", playerId: player2.id })

  // We're now in player1's UNTAP step
  expect(game.currentStep).toBe(Step.UNTAP)
  expect(game.currentPlayerId).toBe(player1.id)

  // Summoning sickness should now be cleared
  creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.hasSummoningSickness).toBe(false)
})

// ============================================================================
// Haste: Bypasses Summoning Sickness
// ============================================================================

test("creature with Haste can attack the turn it enters", () => {
  const { game, player1 } = createStartedGame()

  const hasteCreature = createTestCreatureWithHaste(
    player1.id,
    "haste-creature-1",
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, hasteCreature)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Haste creature should be able to attack (no error)
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: hasteCreature.instanceId,
  })

  const creatureState = game.getCreatureState(hasteCreature.instanceId)
  expect(creatureState.isAttacking).toBe(true)
  // Note: hasSummoningSickness might still be true, but Haste bypasses the check
})

test("creature with Haste can use tap abilities the turn it enters", () => {
  const { game, player1 } = createStartedGame()

  // Create a creature with Haste and a tap ability
  const hasteCreature: CardInstance = {
    instanceId: "haste-tapper",
    definition: {
      id: "haste-tapper-def",
      name: "Haste Tapper",
      types: ["CREATURE"],
      staticAbilities: [StaticAbilities.HASTE],
      power: 1,
      toughness: 1,
      activatedAbility: {
        cost: {
          type: "TAP",
        },
        effect: (game) => {
          // Simple effect: add 1 mana (just to test the tap cost works)
          game.addMana(player1.id, "R", 1)
        },
      },
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefield(game, player1.id, hasteCreature)

  advanceToStep(game, Step.FIRST_MAIN)

  // Verify player1 has priority before activating ability
  // In FIRST_MAIN, priority starts with the active player
  expect(game.currentPlayerId).toBe(player1.id)

  // Haste creature should be able to activate tap ability (no error)
  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: hasteCreature.instanceId,
  })

  const creatureState = game.getCreatureState(hasteCreature.instanceId)
  expect(creatureState.isTapped).toBe(true)
})

// ============================================================================
// Summoning Sickness: Tap Ability Restriction
// ============================================================================

test("creature with summoning sickness cannot use tap ability", () => {
  const { game, player1 } = createStartedGame()

  // Create a creature with a tap ability
  const creature: CardInstance = {
    instanceId: "tapper",
    definition: {
      id: "tapper-def",
      name: "Tapper",
      types: ["CREATURE"],
      power: 1,
      toughness: 1,
      activatedAbility: {
        cost: {
          type: "TAP",
        },
        effect: (game) => {
          game.addMana(player1.id, "R", 1)
        },
      },
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  advanceToStep(game, Step.FIRST_MAIN)

  // Try to activate tap ability - should throw CreatureHasSummoningSicknessError
  expect(() => {
    game.apply({
      type: "ACTIVATE_ABILITY",
      playerId: player1.id,
      permanentId: creature.instanceId,
    })
  }).toThrow(/has summoning sickness/)
})

test("creature can use tap ability after summoning sickness is cleared", () => {
  const { game, player1, player2 } = createStartedGameWithDecks()

  // Create a creature with a tap ability
  const creature: CardInstance = {
    instanceId: "tapper",
    definition: {
      id: "tapper-def",
      name: "Tapper",
      types: ["CREATURE"],
      power: 1,
      toughness: 1,
      activatedAbility: {
        cost: {
          type: "TAP",
        },
        effect: (game) => {
          game.addMana(player1.id, "R", 1)
        },
      },
    },
    ownerId: player1.id,
  }

  addCreatureToBattlefieldWithSummoningSickness(game, player1.id, creature)

  // End turn and start next turn
  advanceToStep(game, Step.CLEANUP)

  // Advance through CLEANUP to enter player2's turn
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Now we're in player2's UNTAP, advance through their entire turn
  while (
    game.currentStep !== Step.CLEANUP ||
    game.currentPlayerId !== player2.id
  ) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  // Advance from player2's CLEANUP to player1's UNTAP
  game.apply({ type: "ADVANCE_STEP", playerId: player2.id })

  // Advance to FIRST_MAIN
  advanceToStep(game, Step.FIRST_MAIN)

  // Verify player1 has priority
  expect(game.currentPlayerId).toBe(player1.id)

  // Now creature should be able to activate tap ability (no error)
  game.apply({
    type: "ACTIVATE_ABILITY",
    playerId: player1.id,
    permanentId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isTapped).toBe(true)
})
