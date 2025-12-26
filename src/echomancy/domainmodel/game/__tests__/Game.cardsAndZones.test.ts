import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { CardIsNotLandError, CardNotFoundInHandError } from "../GameErrors"
import { Step } from "../Steps"
import {
  advanceToStep,
  createGameInMainPhaseWithLand,
  createStartedGame,
} from "./helpers"

test("it moves a land card from hand to battlefield when playing a land", () => {
  const { game, player1, land } = createGameInMainPhaseWithLand()

  const stateBefore = game.getPlayerState(player1.id)
  const playedCard = stateBefore.hand.cards[0]

  expect(stateBefore.hand.cards).toHaveLength(1)
  expect(stateBefore.battlefield.cards).toHaveLength(0)
  expect(playedCard.instanceId).toBe(land.instanceId)

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land.instanceId,
  })

  const stateAfter = game.getPlayerState(player1.id)

  expect(stateAfter.hand.cards).toHaveLength(0)
  expect(stateAfter.battlefield.cards).toHaveLength(1)
  expect(stateAfter.battlefield.cards[0].instanceId).toBe(land.instanceId)
})

test("it throws error when trying to play a land that is not in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: "non-existent-card-id",
    })
  }).toThrow(CardNotFoundInHandError)
})

test("it throws error when trying to play a non-land card", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Insert a spell card in hand
  const playerState = game.getPlayerState(player1.id)
  const spellCard: CardInstance = {
    instanceId: "spell-card-instance",
    definition: {
      id: "spell",
      name: "Test Spell",
      types: ["INSTANT"],
    },
    ownerId: player1.id,
  }
  playerState.hand.cards.push(spellCard)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: spellCard.instanceId,
    })
  }).toThrow(CardIsNotLandError)
})

test("it moves the same card instance to the battlefield", () => {
  const { game, player1, land } = createGameInMainPhaseWithLand()

  const stateBefore = game.getPlayerState(player1.id)
  const originalCard = stateBefore.hand.cards[0]

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land.instanceId,
  })

  const stateAfter = game.getPlayerState(player1.id)
  const movedCard = stateAfter.battlefield.cards[0]

  expect(movedCard.instanceId).toBe(originalCard.instanceId)
  expect(movedCard.definition.id).toBe(originalCard.definition.id)
  expect(movedCard.ownerId).toBe(originalCard.ownerId)
})
