import { v4 as uuidv4 } from "uuid"
import { expect, test } from "vitest"
import { Game } from "./Game"
import { Player } from "./Player"
import { Step } from "./Steps"

test("it returns no allowed actions for non-current player", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  const actions = game.getAllowedActionsFor(player2.id)
  expect(actions).toEqual([])
})

test("it allows advance step and end turn for current player", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toEqual(["ADVANCE_STEP", "END_TURN"])
})

test("it returns no allowed actions during cleanup step", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  while (game.currentStep !== Step.CLEANUP) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  const actions = game.getAllowedActionsFor(game.currentPlayerId)
  expect(actions).toEqual([])
})
