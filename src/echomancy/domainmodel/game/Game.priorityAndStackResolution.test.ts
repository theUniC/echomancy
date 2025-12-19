import { expect, test } from "vitest"
import {
  advanceToStep,
  castSpellInMainPhase,
  createStartedGame,
} from "./__tests__/helpers"
import { InvalidPlayerActionError } from "./GameErrors"
import { Step } from "./Steps"

test("it allows PASS_PRIORITY only when there is a spell on the stack", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  let actions = game.getAllowedActionsFor(player1.id)
  expect(actions).not.toContain("PASS_PRIORITY")

  castSpellInMainPhase(game, player1.id)

  actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("PASS_PRIORITY")
})

test("it does not resolve the stack after a single priority pass", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  castSpellInMainPhase(game, player1.id)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  const stack = game.getStack()
  expect(stack).toHaveLength(1)

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(0)
})

test("it resolves the top of the stack after both players pass priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  castSpellInMainPhase(game, player1.id)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  const stack = game.getStack()
  expect(stack).toHaveLength(0)

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)

  const stateAfter = game.getPlayerState(player1.id)
  expect(stateAfter.battlefield.cards).toHaveLength(0)
})

test("it moves the same spell instance from stack to graveyard", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = castSpellInMainPhase(game, player1.id)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard[0].instanceId).toBe(spellCard.instanceId)
  expect(graveyard[0].definition.id).toBe(spellCard.definition.id)
})

test("it resets priority to current player after stack resolution", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  castSpellInMainPhase(game, player1.id)

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).not.toContain("PASS_PRIORITY")
  expect(actions).toContain("ADVANCE_STEP")
  expect(actions).toContain("END_TURN")
})

test("it throws error when non-priority player passes priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  castSpellInMainPhase(game, player1.id)

  expect(() => {
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })
  }).toThrow(InvalidPlayerActionError)
})
