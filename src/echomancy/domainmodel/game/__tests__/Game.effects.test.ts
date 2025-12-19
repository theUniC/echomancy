import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { Effect } from "../../effects/Effect"
import type { EffectContext } from "../../effects/EffectContext"
import { NoOpEffect } from "../../effects/impl/NoOpEffect"
import type { Game } from "../Game"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

test("it executes effect when resolving spell from stack", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let effectExecuted = false
  const testEffect: Effect = {
    resolve(_game: Game, _context: EffectContext) {
      effectExecuted = true
    },
  }

  const spellCard: CardInstance = {
    instanceId: "effect-spell-instance",
    definition: {
      id: "effect-spell",
      name: "Effect Spell",
      type: "SPELL",
      effect: testEffect,
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  expect(effectExecuted).toBe(false)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  expect(effectExecuted).toBe(true)
})

test("it resolves spell without effect (no crash)", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "no-effect-spell-instance",
    definition: {
      id: "no-effect-spell",
      name: "No Effect Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(spellCard.instanceId)
})

test("it executes NoOpEffect without crashing", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "noop-spell-instance",
    definition: {
      id: "noop-spell",
      name: "No-Op Spell",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
})

test("effect receives correct game and source card", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let receivedGame: Game | null = null
  let receivedSource: CardInstance | null = null
  let receivedControllerId: string | null = null

  const testEffect: Effect = {
    resolve(g: Game, context: EffectContext) {
      receivedGame = g
      receivedSource = context.source
      receivedControllerId = context.controllerId
    },
  }

  const spellCard: CardInstance = {
    instanceId: "param-test-spell",
    definition: {
      id: "param-test",
      name: "Param Test Spell",
      type: "SPELL",
      effect: testEffect,
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  expect(receivedGame).toBe(game)
  expect(receivedSource).toBe(spellCard)
  expect(receivedControllerId).toBe(player1.id)
  expect(spellCard.ownerId).toBe(player1.id)
})

test("effect is executed before card moves to graveyard", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let graveyardSizeWhenEffectRan = -1

  const testEffect: Effect = {
    resolve(g: Game, context: EffectContext) {
      const graveyard = g.getGraveyard(context.controllerId)
      graveyardSizeWhenEffectRan = graveyard.length
    },
  }

  const spellCard: CardInstance = {
    instanceId: "timing-test-spell",
    definition: {
      id: "timing-test",
      name: "Timing Test Spell",
      type: "SPELL",
      effect: testEffect,
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  expect(graveyardSizeWhenEffectRan).toBe(0)

  const graveyardAfter = game.getGraveyard(player1.id)
  expect(graveyardAfter).toHaveLength(1)
})
