import { expect, test } from "vitest"
import { advanceToStep, createStartedGame } from "./__tests__/helpers"
import {
  InvalidPlayerActionError,
  InvalidPlayLandStepError,
  LandLimitExceededError,
} from "./GameErrors"
import { Step } from "./Steps"

test("it allows the current player to play a land in first main phase", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: dummyLandInstanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})

test("it throws error when trying to play land outside main phases", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: dummyLandInstanceId,
    })
  }).toThrow(InvalidPlayLandStepError)

  advanceToStep(game, Step.DRAW)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: dummyLandInstanceId,
    })
  }).toThrow(InvalidPlayLandStepError)
})

test("it throws error when non-current player tries to play a land", () => {
  const { game, player2, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player2.id,
      cardId: dummyLandInstanceId,
    })
  }).toThrow(InvalidPlayerActionError)
})

test("it does not allow playing more than one land per turn", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: dummyLandInstanceId,
  })

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: dummyLandInstanceId,
    })
  }).toThrow(LandLimitExceededError)
})

test("it removes PLAY_LAND from allowed actions after playing a land", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: dummyLandInstanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).toEqual(["ADVANCE_STEP", "END_TURN"])
})

test("it allows playing a land again on the next turn", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: dummyLandInstanceId,
  })
  game.apply({ type: "END_TURN", playerId: game.currentPlayerId })
  advanceToStep(game, Step.FIRST_MAIN)

  const actions = game.getAllowedActionsFor(game.currentPlayerId)
  expect(actions).toContain("PLAY_LAND")
})

test("it allows playing land in second main phase", () => {
  const { game, player1, dummyLandInstanceId } = createStartedGame()
  advanceToStep(game, Step.SECOND_MAIN)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: dummyLandInstanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})
