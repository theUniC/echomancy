import { expect, test } from "vitest"
import {
  InvalidPlayerActionError,
  InvalidPlayLandStepError,
  LandLimitExceededError,
} from "../GameErrors"
import { Step } from "../Steps"
import {
  addTestLandToHand,
  advanceToStep,
  createGameInMainPhaseWithLand,
  createStartedGame,
} from "./helpers"

test("it allows the current player to play a land in first main phase", () => {
  const { game, player1, land } = createGameInMainPhaseWithLand()

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land.instanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})

test("it throws error when trying to play land outside main phases", () => {
  const { game, player1 } = createStartedGame()
  const land = addTestLandToHand(game, player1.id)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: land.instanceId,
    })
  }).toThrow(InvalidPlayLandStepError)

  advanceToStep(game, Step.DRAW)

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: land.instanceId,
    })
  }).toThrow(InvalidPlayLandStepError)
})

test("it throws error when non-current player tries to play a land", () => {
  const { game, player2, land } = createGameInMainPhaseWithLand()

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player2.id,
      cardId: land.instanceId,
    })
  }).toThrow(InvalidPlayerActionError)
})

test("it does not allow playing more than one land per turn", () => {
  const { game, player1 } = createStartedGame()
  const land1 = addTestLandToHand(game, player1.id, "land-1")
  const land2 = addTestLandToHand(game, player1.id, "land-2")
  advanceToStep(game, Step.FIRST_MAIN)

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land1.instanceId,
  })

  expect(() => {
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: land2.instanceId,
    })
  }).toThrow(LandLimitExceededError)
})

test("it removes PLAY_LAND from allowed actions after playing a land", () => {
  const { game, player1, land } = createGameInMainPhaseWithLand()

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land.instanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).toEqual(["ADVANCE_STEP", "END_TURN"])
})

test("it allows playing a land again on the next turn", () => {
  const { game, player1 } = createStartedGame()
  const land1 = addTestLandToHand(game, player1.id, "land-1")
  const _land2 = addTestLandToHand(game, player1.id, "land-2")
  advanceToStep(game, Step.FIRST_MAIN)

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land1.instanceId,
  })
  game.apply({ type: "END_TURN", playerId: game.currentPlayerId })
  advanceToStep(game, Step.FIRST_MAIN)

  const actions = game.getAllowedActionsFor(game.currentPlayerId)
  expect(actions).toContain("PLAY_LAND")
})

test("it allows playing land in second main phase", () => {
  const { game, player1 } = createStartedGame()
  const land = addTestLandToHand(game, player1.id)
  advanceToStep(game, Step.SECOND_MAIN)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("PLAY_LAND")

  game.apply({
    type: "PLAY_LAND",
    playerId: player1.id,
    cardId: land.instanceId,
  })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})
