import { expect, test } from "vitest"
import {
  CardIsNotSpellError,
  CardNotFoundInHandError,
  InvalidCastSpellStepError,
  InvalidPlayerActionError,
} from "../GameErrors"
import { Step } from "../Steps"
import {
  addSpellToHand,
  addTestLandToHand,
  advanceToStep,
  assertSpellAt,
  createStartedGame,
  createTestSpell,
} from "./helpers"

test("it moves a spell card from hand to stack when casting a spell", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stateAfter = game.getPlayerState(player1.id)
  const stack = game.getStack()

  expect(stateAfter.hand.cards).toHaveLength(0) // Spell moved to stack
  expect(stack).toHaveLength(1)
  expect(stateAfter.battlefield.cards).toHaveLength(0)
})

test("it pushes the same card instance onto the stack", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stack = game.getStack()
  const spell = assertSpellAt(stack, 0)

  expect(spell.card.instanceId).toBe(spellCard.instanceId)
  expect(spell.card.definition.id).toBe(spellCard.definition.id)
  expect(spell.controllerId).toBe(player1.id)
})

test("it throws error when trying to cast spell outside main phases", () => {
  const { game, player1 } = createStartedGame()

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })
  }).toThrow(InvalidCastSpellStepError)
})

test("it throws error when non-current player tries to cast spell", () => {
  const { game, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player2.id)
  addSpellToHand(game, player2.id, spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player2.id,
      cardId: spellCard.instanceId,
      targets: [],
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
      targets: [],
    })
  }).toThrow(CardNotFoundInHandError)
})

test("it throws error when trying to cast a non-spell card", () => {
  const { game, player1 } = createStartedGame()
  const land = addTestLandToHand(game, player1.id)
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: land.instanceId,
      targets: [],
    })
  }).toThrow(CardIsNotSpellError)
})

test("it does not show CAST_SPELL action if player has no spells in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).not.toContain("CAST_SPELL")
})

test("it shows CAST_SPELL action when player has spell in hand during main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")
})

test("it allows casting spell in second main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.SECOND_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stateAfter = game.getPlayerState(player1.id)
  expect(stateAfter.hand.cards).not.toContainEqual(spellCard)
})
