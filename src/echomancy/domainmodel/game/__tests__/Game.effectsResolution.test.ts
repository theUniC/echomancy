import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { Effect } from "../../effects/Effect"
import { NoOpEffect } from "../../effects/impl/NoOpEffect"
import type { Game } from "../Game"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

test("it resolves a spell with NoOpEffect and moves it to graveyard", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "noop-spell",
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

  expect(game.getStack()).toHaveLength(1)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  const stack = game.getStack()
  expect(stack).toHaveLength(0)

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(spellCard.instanceId)

  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(0)
})

test("it executes the effect before moving the spell to graveyard", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const observableEffect: Effect = {
    resolve(g: Game, source: CardInstance) {
      const dummyCard: CardInstance = {
        instanceId: "observable-card",
        definition: {
          id: "observable",
          name: "Observable Card",
          type: "SPELL",
        },
        ownerId: source.ownerId,
      }
      const playerState = g.getPlayerState(source.ownerId)
      playerState.hand.cards.push(dummyCard)
    },
  }

  const spellCard: CardInstance = {
    instanceId: "observable-spell",
    definition: {
      id: "observable-spell",
      name: "Observable Spell",
      type: "SPELL",
      effect: observableEffect,
    },
    ownerId: player1.id,
  }

  const handSizeBefore = game.getPlayerState(player1.id).hand.cards.length

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

  const handSizeAfter = game.getPlayerState(player1.id).hand.cards.length
  expect(handSizeAfter).toBe(handSizeBefore + 1)

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(spellCard.instanceId)
})

test("it resolves a spell without effect without throwing", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard: CardInstance = {
    instanceId: "no-effect-spell",
    definition: {
      id: "no-effect-spell",
      name: "No Effect Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard)

  expect(() => {
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
  }).not.toThrow()

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(spellCard.instanceId)
})

test("it resets priority to current player after resolving an effect", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard1: CardInstance = {
    instanceId: "spell-1",
    definition: {
      id: "spell-1",
      name: "Spell 1",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  const spellCard2: CardInstance = {
    instanceId: "spell-2",
    definition: {
      id: "spell-2",
      name: "Spell 2",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  addSpellToHand(game, player1.id, spellCard1)
  addSpellToHand(game, player1.id, spellCard2)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard1.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  const actionsAfterResolution = game.getAllowedActionsFor(player1.id)
  expect(actionsAfterResolution).not.toContain("PASS_PRIORITY")
  expect(actionsAfterResolution).toContain("CAST_SPELL")
  expect(actionsAfterResolution).toContain("ADVANCE_STEP")
  expect(actionsAfterResolution).toContain("END_TURN")
})

test("it does not allow effects to resolve additional spells implicitly", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard1: CardInstance = {
    instanceId: "spell-1",
    definition: {
      id: "spell-1",
      name: "Spell 1",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player1.id,
  }

  const spellCard2: CardInstance = {
    instanceId: "spell-2",
    definition: {
      id: "spell-2",
      name: "Spell 2",
      type: "SPELL",
      effect: new NoOpEffect(),
    },
    ownerId: player2.id,
  }

  addSpellToHand(game, player1.id, spellCard1)
  addSpellToHand(game, player2.id, spellCard2)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard1.instanceId,
  })

  game.apply({
    type: "CAST_SPELL",
    playerId: player2.id,
    cardId: spellCard2.instanceId,
  })

  const stackBefore = game.getStack()
  expect(stackBefore).toHaveLength(2)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  const stackAfter = game.getStack()
  expect(stackAfter).toHaveLength(1)

  const graveyard2 = game.getGraveyard(player2.id)
  expect(graveyard2).toHaveLength(1)
  expect(graveyard2[0].instanceId).toBe(spellCard2.instanceId)

  const graveyard1 = game.getGraveyard(player1.id)
  expect(graveyard1).toHaveLength(0)
})
