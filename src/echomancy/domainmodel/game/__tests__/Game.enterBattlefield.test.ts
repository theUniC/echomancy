import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { EffectContext } from "../../effects/EffectContext"
import type { Target } from "../../targets/Target"
import { ZoneNames } from "../../zones/Zone"
import type { Game } from "../Game"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

/**
 * enterBattlefield MVP Tests
 *
 * These tests verify that ALL paths to the battlefield go through
 * a centralized enterBattlefield function that:
 * 1. Moves the permanent to battlefield
 * 2. Initializes creature state if applicable
 * 3. Executes ETB effects consistently
 *
 * TODO: ETB with targets is not implemented yet
 * TODO: ETB does not use the stack as separate trigger
 * TODO: Full triggered abilities will come later
 * TODO: Replacement effects are not implemented
 * TODO: Complete Last Known Information is not implemented
 */

// TEST 1: Permanent from stack enters via enterBattlefield
test("permanent resolved from stack enters via enterBattlefield", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbExecuted = false

  const creatureCard: CardInstance = {
    instanceId: "creature-from-stack",
    definition: {
      id: "creature-from-stack",
      name: "Creature from Stack",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: "ZONE_CHANGED",
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (_g: Game, _context: EffectContext) => {
            etbExecuted = true
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, creatureCard)

  // Cast the creature
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureCard.instanceId,
    targets: [],
  })

  // Resolve the stack
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify the permanent is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].instanceId).toBe(creatureCard.instanceId)

  // Verify ETB was executed
  expect(etbExecuted).toBe(true)
})

// TEST 2: Land played enters via enterBattlefield
test("land played from hand enters via enterBattlefield and executes ETB", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbExecuted = false

  const landWithETB: CardInstance = {
    instanceId: "land-with-etb",
    definition: {
      id: "land-with-etb",
      name: "Land with ETB",
      types: ["LAND"],
      triggers: [
        {
          eventType: "ZONE_CHANGED",
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (_g: Game, _context: EffectContext) => {
            etbExecuted = true
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  // Add land to hand
  game.getPlayerState(player1.id).hand.cards.push(landWithETB)

  // Play the land
  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: landWithETB.instanceId,
  })

  // Verify the land is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].instanceId).toBe(landWithETB.instanceId)

  // Verify ETB was executed (this is the key improvement)
  expect(etbExecuted).toBe(true)
})

// TEST 3: ETB executes exactly once
test("ETB executes exactly once when permanent enters", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbExecutionCount = 0

  const permanentCard: CardInstance = {
    instanceId: "permanent-etb-once",
    definition: {
      id: "permanent-etb-once",
      name: "Permanent ETB Once",
      types: ["ARTIFACT"],
      triggers: [
        {
          eventType: "ZONE_CHANGED",
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (_g: Game, _context: EffectContext) => {
            etbExecutionCount++
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, permanentCard)

  // Cast and resolve
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: permanentCard.instanceId,
    targets: [],
  })

  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify ETB executed exactly once
  expect(etbExecutionCount).toBe(1)
})

// TEST 4: ETB does NOT inherit targets from spell
test("ETB receives empty targets, does not inherit from spell", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbReceivedTargets: Target[] | undefined

  const creatureCard: CardInstance = {
    instanceId: "creature-with-targets",
    definition: {
      id: "creature-with-targets",
      name: "Creature with Targets",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: "ZONE_CHANGED",
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (_g: Game, context: EffectContext) => {
            // Capture the targets received by ETB
            etbReceivedTargets = context.targets
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, creatureCard)

  // Cast the spell WITH a target (targeting player2)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureCard.instanceId,
    targets: [
      {
        kind: "PLAYER",
        playerId: player2.id,
      },
    ],
  })

  // Resolve
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify ETB received empty targets (did not inherit the spell's target)
  expect(etbReceivedTargets).toBeDefined()
  expect(etbReceivedTargets).toEqual([])
})

// TEST 5: Creature state is initialized when creature enters
test("creature state is initialized correctly on battlefield entry", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creatureCard: CardInstance = {
    instanceId: "creature-state-init",
    definition: {
      id: "creature-state-init",
      name: "Creature State Init",
      types: ["CREATURE"],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, creatureCard)

  // Cast and resolve
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureCard.instanceId,
    targets: [],
  })

  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify creature is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)

  // Verify creature state was initialized
  const creatureState = game.getCreatureState(creatureCard.instanceId)
  expect(creatureState).toBeDefined()
  expect(creatureState?.isTapped).toBe(false)
  expect(creatureState?.isAttacking).toBe(false)
  expect(creatureState?.hasAttackedThisTurn).toBe(false)
})

// TEST 6: Non-creature permanent does NOT get creature state
test("non-creature permanent does not get creature state", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const artifactCard: CardInstance = {
    instanceId: "artifact-no-state",
    definition: {
      id: "artifact-no-state",
      name: "Artifact No State",
      types: ["ARTIFACT"],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, artifactCard)

  // Cast and resolve
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: artifactCard.instanceId,
    targets: [],
  })

  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify artifact is on battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)

  // Verify NO creature state was created
  // getCreatureState throws for non-creatures, which is the expected behavior
  expect(() => game.getCreatureState(artifactCard.instanceId)).toThrow()
})

// TEST 7: ETB context has correct source and controllerId
test("ETB context contains correct source and controllerId", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let capturedContext: EffectContext | undefined

  const creatureCard: CardInstance = {
    instanceId: "creature-context-check",
    definition: {
      id: "creature-context-check",
      name: "Creature Context Check",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: "ZONE_CHANGED",
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (_g: Game, context: EffectContext) => {
            capturedContext = context
          },
        },
      ],
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, creatureCard)

  // Cast and resolve
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureCard.instanceId,
    targets: [],
  })

  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
  game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

  // Verify context was captured
  expect(capturedContext).toBeDefined()

  // Verify source is the permanent that entered
  expect(capturedContext?.source.instanceId).toBe(creatureCard.instanceId)

  // Verify controllerId is correct
  expect(capturedContext?.controllerId).toBe(player1.id)

  // Verify targets are empty
  expect(capturedContext?.targets).toEqual([])
})

// TEST 8 (TODO): Blink future possibility (conceptual)
test.todo(
  "blink: permanent that leaves and re-enters battlefield triggers ETB again",
)
