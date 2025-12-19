import { expect, test } from "vitest"
import type { CardInstance } from "../cards/CardInstance"
import { advanceToStep, createStartedGame } from "./__tests__/helpers"
import {
  CardIsNotSpellError,
  CardNotFoundInHandError,
  InvalidCastSpellStepError,
  InvalidPlayerActionError,
} from "./GameErrors"
import { Step } from "./Steps"

test("it moves a spell card from hand to stack when casting a spell", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  const stateAfter = game.getPlayerState(player1.id)
  const stack = game.getStack()

  expect(stateAfter.hand.cards).toHaveLength(1) // Only the dummy land remains
  expect(stack).toHaveLength(1)
  expect(stateAfter.battlefield.cards).toHaveLength(0)
})

test("it pushes the same card instance onto the stack", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  const stack = game.getStack()

  expect(stack[0].card.instanceId).toBe(spellCard.instanceId)
  expect(stack[0].card.definition.id).toBe(spellCard.definition.id)
  expect(stack[0].controllerId).toBe(player1.id)
})

test("it throws error when trying to cast spell outside main phases", () => {
  const { game, player1 } = createStartedGame()

  // Add a spell card to hand
  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
    })
  }).toThrow(InvalidCastSpellStepError)
})

test("it throws error when non-current player tries to cast spell", () => {
  const { game, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Add a spell card to player2's hand
  const playerState = game.getPlayerState(player2.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player2.id,
  }
  playerState.hand.cards.push(spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player2.id,
      cardId: spellCard.instanceId,
    })
  }).toThrow(InvalidPlayerActionError)
})

test("it throws error when trying to cast a card that is not in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: "non-existent-card",
    })
  }).toThrow(CardNotFoundInHandError)
})

test("it throws error when trying to cast a non-spell card", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: dummyLandInstanceId,
    })
  }).toThrow(CardIsNotSpellError)
})

test("it does not show CAST_SPELL action if player has no spells in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player only has a land in hand (from createStartedGame)
  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).not.toContain("CAST_SPELL")
})

test("it shows CAST_SPELL action when player has spell in hand during main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Add a spell to hand
  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")
})

test("it allows casting spell in second main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.SECOND_MAIN)

  // Add a spell to hand
  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "test-spell-instance",
    definition: {
      id: "test-spell",
      name: "Test Spell",
      type: "SPELL",
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
  })

  const stateAfter = game.getPlayerState(player1.id)
  expect(stateAfter.hand.cards).not.toContainEqual(spellCard)
})
