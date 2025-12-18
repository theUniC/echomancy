import { validate as isValidUUID, v4 as uuidv4 } from "uuid"
import { expect, test } from "vitest"
import { advanceToStep, createStartedGame } from "./__tests__/helpers"
import { Game } from "./Game"
import {
  InvalidEndTurnError,
  InvalidPlayerActionError,
  InvalidPlayerCountError,
  InvalidStartingPlayerError,
} from "./GameErrors"
import { Player } from "./Player"

test("it can be instantiated", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const playersById = new Map([
    [player1.id, player1],
    [player2.id, player2],
  ])
  const turnOrder = [player1.id, player2.id]
  const playerStates = new Map([
    [player1.id, { hand: { cards: [] }, battlefield: { cards: [] } }],
    [player2.id, { hand: { cards: [] }, battlefield: { cards: [] } }],
  ])
  const game = new Game(
    "game-id",
    playersById,
    turnOrder,
    player1.id,
    "UNTAP",
    playerStates,
  )

  expect(game).toBeInstanceOf(Game)
  expect(game.id).toBe("game-id")
  expect(game.getCurrentPlayer()).toBe(player1)
  expect(game.currentPlayerId).toBe(player1.id)
  expect(game.currentStep).toBe("UNTAP")
})

test("it accepts an id parameter when using start factory", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const gameId = uuidv4()
  const game = Game.start({
    id: gameId,
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  expect(game.id).toBe(gameId)
  expect(isValidUUID(game.id)).toBe(true)
})

test("it starts at UNTAP step when using start factory", () => {
  const { game } = createStartedGame()

  expect(game.currentStep).toBe("UNTAP")
})

test("it sets the starting player correctly", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player2.id,
  })

  expect(game.currentPlayerId).toBe(player2.id)
})

test("it stores player instances", () => {
  const { game, player1 } = createStartedGame()

  expect(game.getCurrentPlayer()).toBe(player1)
  expect(isValidUUID(player1.id)).toBe(true)
})

test("it can apply AdvanceStep action", () => {
  const { game, player1 } = createStartedGame()

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  expect(game.currentStep).toBe("UPKEEP")
})

test("it throws error when non-current player tries to advance step", () => {
  const { game, player2 } = createStartedGame()

  expect(() => {
    game.apply({ type: "ADVANCE_STEP", playerId: player2.id })
  }).toThrow(InvalidPlayerActionError)
})

test("it advances to next player when completing a turn", () => {
  const { game, player1, player2 } = createStartedGame()

  advanceToStep(game, "CLEANUP")
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  expect(game.currentStep).toBe("UNTAP")
  expect(game.currentPlayerId).toBe(player2.id)
})

test("it validates starting player is in player list", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  expect(() => {
    Game.start({
      id: uuidv4(),
      players: [player1, player2],
      startingPlayerId: "invalid-id",
    })
  }).toThrow(InvalidStartingPlayerError)
})

test("it requires at least 2 players", () => {
  const player1 = new Player("p1")

  expect(() => {
    Game.start({
      id: uuidv4(),
      players: [player1],
      startingPlayerId: player1.id,
    })
  }).toThrow(InvalidPlayerCountError)
})

test("it can end turn from any step except CLEANUP", () => {
  const { game, player1, player2 } = createStartedGame()

  advanceToStep(game, "DRAW")

  game.apply({ type: "END_TURN", playerId: player1.id })

  expect(game.currentStep).toBe("UNTAP")
  expect(game.currentPlayerId).toBe(player2.id)
})

test("it throws error when non-current player tries to end turn", () => {
  const { game, player2 } = createStartedGame()

  expect(() => {
    game.apply({ type: "END_TURN", playerId: player2.id })
  }).toThrow(InvalidPlayerActionError)
})

test("it throws error when trying to end turn from CLEANUP", () => {
  const { game } = createStartedGame()

  advanceToStep(game, "CLEANUP")

  expect(() => {
    game.apply({ type: "END_TURN", playerId: game.currentPlayerId })
  }).toThrow(InvalidEndTurnError)
})

test("it advances through all remaining steps when ending turn", () => {
  const { game, player1, player2 } = createStartedGame()

  expect(game.currentStep).toBe("UNTAP")
  expect(game.currentPlayerId).toBe(player1.id)

  game.apply({ type: "END_TURN", playerId: player1.id })

  expect(game.currentStep).toBe("UNTAP")
  expect(game.currentPlayerId).toBe(player2.id)
})
