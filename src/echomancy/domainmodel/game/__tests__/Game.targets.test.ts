import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { Effect } from "../../effects/Effect"
import type { EffectContext } from "../../effects/EffectContext"
import { NoOpEffect } from "../../effects/impl/NoOpEffect"
import type { Target } from "../../targets/Target"
import type { Game } from "../Game"
import { InvalidPlayerActionError } from "../GameErrors"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

test("it requires explicit targets when casting a spell", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "spell-no-targets",
    definition: {
      id: "spell-no-targets",
      name: "Spell No Targets",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })
  }).not.toThrow()

  const stack = game.getStack()
  expect(stack).toHaveLength(1)
  expect(stack[0].targets).toEqual([])
})

test("it stores targets on the stack when casting a spell", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "spell-with-target",
    definition: {
      id: "spell-with-target",
      name: "Spell With Target",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  const target: Target = {
    kind: "PLAYER",
    playerId: player2.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [target],
  })

  const stack = game.getStack()
  expect(stack).toHaveLength(1)
  expect(stack[0].targets).toEqual([target])
  expect(stack[0].targets[0].kind).toBe("PLAYER")
  expect(stack[0].targets[0].playerId).toBe(player2.id)
})

test("it throws when casting a spell with an invalid PLAYER target", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "spell-invalid-target",
    definition: {
      id: "spell-invalid-target",
      name: "Spell Invalid Target",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  const invalidTarget: Target = {
    kind: "PLAYER",
    playerId: "non-existent-player",
  }

  addSpellToHand(game, player1.id, spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [invalidTarget],
    })
  }).toThrow(InvalidPlayerActionError)
})

test("it passes targets to the effect through EffectContext", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let receivedTargets: Target[] | null = null

  const testEffect: Effect = {
    resolve(_game: Game, context: EffectContext) {
      receivedTargets = context.targets
    },
  }

  const spellCard: CardInstance = {
    instanceId: "spell-test-effect",
    definition: {
      id: "spell-test-effect",
      name: "Spell Test Effect",
      type: "SPELL",
      effect: testEffect,
    },
    ownerId: player1.id,
  }

  const target: Target = {
    kind: "PLAYER",
    playerId: player2.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [target],
  })

  expect(receivedTargets).toBeNull()

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  expect(receivedTargets).not.toBeNull()
  expect(receivedTargets).toHaveLength(1)

  // TypeScript narrowing: we know it's not null after the assertions above
  const targets = receivedTargets as unknown as Target[]
  const firstTarget = targets[0]
  expect(firstTarget.kind).toBe("PLAYER")
  if (firstTarget.kind === "PLAYER") {
    expect(firstTarget.playerId).toBe(player2.id)
  }
})

test("it does not mutate targets during resolution", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "spell-immutable-targets",
    definition: {
      id: "spell-immutable-targets",
      name: "Spell Immutable Targets",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  const originalTarget: Target = {
    kind: "PLAYER",
    playerId: player2.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [originalTarget],
  })

  const stackBefore = game.getStack()
  const targetsBefore = stackBefore[0].targets

  expect(targetsBefore).toHaveLength(1)
  expect(targetsBefore[0].kind).toBe("PLAYER")
  expect(targetsBefore[0].playerId).toBe(player2.id)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  expect(originalTarget.kind).toBe("PLAYER")
  expect(originalTarget.playerId).toBe(player2.id)
})
