import { expect, test } from "vitest"
import { Step } from "./Steps"
import { advanceToStep, createStartedGame } from "./__tests__/helpers"

test("it returns no allowed actions for non-current player", () => {
  const { game, player2 } = createStartedGame()

  const actions = game.getAllowedActionsFor(player2.id)
  expect(actions).toEqual([])
})

test("it allows advance step and end turn for current player", () => {
  const { game, player1 } = createStartedGame()

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toEqual(["ADVANCE_STEP", "END_TURN"])
})

test("it returns no allowed actions during cleanup step", () => {
  const { game } = createStartedGame()
  advanceToStep(game, Step.CLEANUP)

  const actions = game.getAllowedActionsFor(game.currentPlayerId)
  expect(actions).toEqual([])
})
