import { v4 as uuidv4 } from "uuid"
import { expect, test } from "vitest"
import { Game } from "./Game"
import { Player } from "./Player"
import { Step } from "./Steps"
import {
  InvalidPlayerActionError,
  InvalidPlayLandStepError,
  LandLimitExceededError,
} from "./GameErrors"

function createStartedGame() {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  return { game, player1, player2 }
}

test("it allows the current player to play a land in first main phase", () => {
  const { game, player1 } = createStartedGame()

  // Advance to FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})

test("it throws error when trying to play land outside main phases", () => {
  const { game, player1 } = createStartedGame()

  // Try to play land in UNTAP
  expect(() => {
    game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })
  }).toThrow(InvalidPlayLandStepError)

  // Advance to DRAW
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  expect(game.currentStep).toBe(Step.DRAW)

  expect(() => {
    game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })
  }).toThrow(InvalidPlayLandStepError)
})

test("it throws error when non-current player tries to play a land", () => {
  const { game, player2 } = createStartedGame()

  // Advance to FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  expect(() => {
    game.apply({ type: "PLAY_LAND", playerId: player2.id, cardId: "card-1" })
  }).toThrow(InvalidPlayerActionError)
})

test("it does not allow playing more than one land per turn", () => {
  const { game, player1 } = createStartedGame()

  // Advance to FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })

  expect(() => {
    game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-2" })
  }).toThrow(LandLimitExceededError)
})

test("it removes PLAY_LAND from allowed actions after playing a land", () => {
  const { game, player1 } = createStartedGame()

  // Advance to FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  const actionsBefore = game.getAllowedActionsFor(player1.id)
  expect(actionsBefore).toContain("PLAY_LAND")

  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
  expect(actionsAfter).toContain("ADVANCE_STEP")
  expect(actionsAfter).toContain("END_TURN")
})

test("it allows playing a land again on the next turn", () => {
  const { game, player1 } = createStartedGame()

  // Advance to FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })

  // End turn
  game.apply({ type: "END_TURN", playerId: game.currentPlayerId })

  // Advance to next player's FIRST_MAIN
  while (game.currentStep !== Step.FIRST_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  const actions = game.getAllowedActionsFor(game.currentPlayerId)
  expect(actions).toContain("PLAY_LAND")
})

test("it allows playing land in second main phase", () => {
  const { game, player1 } = createStartedGame()

  // Advance to SECOND_MAIN
  while (game.currentStep !== Step.SECOND_MAIN) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("PLAY_LAND")

  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "card-1" })

  const actionsAfter = game.getAllowedActionsFor(player1.id)
  expect(actionsAfter).not.toContain("PLAY_LAND")
})
