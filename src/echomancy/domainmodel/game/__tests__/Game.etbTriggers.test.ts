import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { Effect } from "../../effects/Effect"
import type { EffectContext } from "../../effects/EffectContext"
import type { Game } from "../Game"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

test("it executes ETB when permanent enters the battlefield", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Observable ETB effect that adds a card to the controller's hand
  const etbEffect: Effect = {
    resolve(g: Game, context: EffectContext) {
      const tokenCard: CardInstance = {
        instanceId: "etb-token",
        definition: {
          id: "etb-token",
          name: "ETB Token",
          types: ["CREATURE"],
        },
        ownerId: context.controllerId,
      }
      const playerState = g.getPlayerState(context.controllerId)
      playerState.hand.cards.push(tokenCard)
    },
  }

  const creatureCard: CardInstance = {
    instanceId: "creature-with-etb",
    definition: {
      id: "creature-with-etb",
      name: "Creature with ETB",
      types: ["CREATURE"],
      onEnterBattlefield: etbEffect,
    },
    ownerId: player1.id,
  }

  const handSizeBefore = game.getPlayerState(player1.id).hand.cards.length

  addSpellToHand(game, player1.id, creatureCard)

  // Cast the creature
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creatureCard.instanceId,
    targets: [],
  })

  // Resolve the stack
  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  // Verify the creature is on the battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].instanceId).toBe(creatureCard.instanceId)

  // Verify the ETB effect executed (added a card to hand)
  const handSizeAfter = game.getPlayerState(player1.id).hand.cards.length
  expect(handSizeAfter).toBe(handSizeBefore + 1)

  // Verify the added card is the token
  const hand = game.getPlayerState(player1.id).hand.cards
  const addedCard = hand[hand.length - 1]
  expect(addedCard.instanceId).toBe("etb-token")
})

test("it does NOT execute ETB for instants", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // ETB effect that should NOT execute for instants
  const etbEffect: Effect = {
    resolve(g: Game, context: EffectContext) {
      const tokenCard: CardInstance = {
        instanceId: "etb-token-instant",
        definition: {
          id: "etb-token-instant",
          name: "ETB Token from Instant",
          types: ["CREATURE"],
        },
        ownerId: context.controllerId,
      }
      const playerState = g.getPlayerState(context.controllerId)
      playerState.hand.cards.push(tokenCard)
    },
  }

  const instantCard: CardInstance = {
    instanceId: "instant-with-etb",
    definition: {
      id: "instant-with-etb",
      name: "Instant with ETB",
      types: ["INSTANT"],
      onEnterBattlefield: etbEffect, // ETB defined but should not execute
    },
    ownerId: player1.id,
  }

  const handSizeBefore = game.getPlayerState(player1.id).hand.cards.length

  addSpellToHand(game, player1.id, instantCard)

  // Cast the instant
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: instantCard.instanceId,
    targets: [],
  })

  // Resolve the stack
  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  // Verify the instant is NOT on the battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(0)

  // Verify it's in the graveyard
  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(instantCard.instanceId)

  // Verify the ETB did NOT execute (hand size should be unchanged)
  const handSizeAfter = game.getPlayerState(player1.id).hand.cards.length
  expect(handSizeAfter).toBe(handSizeBefore)
})

test("it does NOT execute ETB for sorceries", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // ETB effect that should NOT execute for sorceries
  const etbEffect: Effect = {
    resolve(g: Game, context: EffectContext) {
      const tokenCard: CardInstance = {
        instanceId: "etb-token-sorcery",
        definition: {
          id: "etb-token-sorcery",
          name: "ETB Token from Sorcery",
          types: ["CREATURE"],
        },
        ownerId: context.controllerId,
      }
      const playerState = g.getPlayerState(context.controllerId)
      playerState.hand.cards.push(tokenCard)
    },
  }

  const sorceryCard: CardInstance = {
    instanceId: "sorcery-with-etb",
    definition: {
      id: "sorcery-with-etb",
      name: "Sorcery with ETB",
      types: ["SORCERY"],
      onEnterBattlefield: etbEffect, // ETB defined but should not execute
    },
    ownerId: player1.id,
  }

  const handSizeBefore = game.getPlayerState(player1.id).hand.cards.length

  addSpellToHand(game, player1.id, sorceryCard)

  // Cast the sorcery
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: sorceryCard.instanceId,
    targets: [],
  })

  // Resolve the stack
  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  // Verify the sorcery is NOT on the battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(0)

  // Verify it's in the graveyard
  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
  expect(graveyard[0].instanceId).toBe(sorceryCard.instanceId)

  // Verify the ETB did NOT execute (hand size should be unchanged)
  const handSizeAfter = game.getPlayerState(player1.id).hand.cards.length
  expect(handSizeAfter).toBe(handSizeBefore)
})

test("it executes ETB after the spell effect", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const executionOrder: string[] = []

  // Spell effect that executes first
  const spellEffect: Effect = {
    resolve(_g: Game, _context: EffectContext) {
      executionOrder.push("SPELL_EFFECT")
    },
  }

  // ETB effect that executes after
  const etbEffect: Effect = {
    resolve(_g: Game, _context: EffectContext) {
      executionOrder.push("ETB_EFFECT")
    },
  }

  const creatureCard: CardInstance = {
    instanceId: "creature-with-both-effects",
    definition: {
      id: "creature-with-both-effects",
      name: "Creature with Both Effects",
      types: ["CREATURE"],
      effect: spellEffect,
      onEnterBattlefield: etbEffect,
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
  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  // Verify the execution order is correct
  expect(executionOrder).toEqual(["SPELL_EFFECT", "ETB_EFFECT"])
})

test("it does not re-trigger ETB on extra phases", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let etbExecutionCount = 0

  // ETB effect that counts how many times it executes
  const etbEffect: Effect = {
    resolve(_g: Game, _context: EffectContext) {
      etbExecutionCount++
    },
  }

  const creatureCard: CardInstance = {
    instanceId: "creature-etb-once",
    definition: {
      id: "creature-etb-once",
      name: "Creature ETB Once",
      types: ["CREATURE"],
      onEnterBattlefield: etbEffect,
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
  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  // Verify ETB executed exactly once
  expect(etbExecutionCount).toBe(1)

  // Advance to second main phase
  game.apply({
    type: "ADVANCE_STEP",
    playerId: player1.id,
  })

  advanceToStep(game, Step.SECOND_MAIN)

  // Verify ETB did not execute again
  expect(etbExecutionCount).toBe(1)

  // Advance to next turn
  game.apply({
    type: "END_TURN",
    playerId: player1.id,
  })

  advanceToStep(game, Step.FIRST_MAIN)

  // Verify ETB still has not executed again
  expect(etbExecutionCount).toBe(1)

  // Verify the creature is still on the battlefield
  const battlefield = game.getPlayerState(player1.id).battlefield.cards
  expect(battlefield).toHaveLength(1)
  expect(battlefield[0].instanceId).toBe(creatureCard.instanceId)
})
