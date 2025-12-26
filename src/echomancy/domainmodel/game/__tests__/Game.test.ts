import { validate as isValidUUID, v4 as uuidv4 } from "uuid"
import { expect, test } from "vitest"
import { Game, GameLifecycleState } from "../Game"
import {
  InvalidEndTurnError,
  InvalidPlayerActionError,
  InvalidPlayerCountError,
  InvalidStartingPlayerError,
} from "../GameErrors"
import { Player } from "../Player"
import { advanceToStep, createStartedGame } from "./helpers"

test("it can be instantiated", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const playersById = new Map([
    [player1.id, player1],
    [player2.id, player2],
  ])
  const turnOrder = [player1.id, player2.id]
  const playerStates = new Map([
    [
      player1.id,
      {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    ],
    [
      player2.id,
      {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    ],
  ])

  const manaPools = new Map([
    [player1.id, { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 }],
    [player2.id, { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 }],
  ])

  const game = new Game(
    "game-id",
    playersById,
    turnOrder,
    player1.id,
    "UNTAP",
    playerStates,
    manaPools,
    GameLifecycleState.STARTED, // Explicitly start in STARTED state for this test
  )

  expect(game).toBeInstanceOf(Game)
  expect(game.id).toBe("game-id")
  expect(game.getCurrentPlayer()).toBe(player1)
  expect(game.currentPlayerId).toBe(player1.id)
  expect(game.currentStep).toBe("UNTAP")
})

test("it accepts an id parameter when using create factory", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const gameId = uuidv4()
  const game = Game.create(gameId)
  game.addPlayer(player1)
  game.addPlayer(player2)
  game.start(player1.id)

  expect(game.id).toBe(gameId)
  expect(isValidUUID(game.id)).toBe(true)
})

test("it starts at UNTAP step after start is called", () => {
  const { game } = createStartedGame()

  expect(game.currentStep).toBe("UNTAP")
})

test("it sets the starting player correctly", () => {
  const player1 = new Player("p1")
  const player2 = new Player("p2")
  const game = Game.create(uuidv4())
  game.addPlayer(player1)
  game.addPlayer(player2)
  game.start(player2.id)

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
  const game = Game.create(uuidv4())
  game.addPlayer(player1)
  game.addPlayer(player2)

  expect(() => {
    game.start("invalid-id")
  }).toThrow(InvalidStartingPlayerError)
})

test("it requires at least 2 players", () => {
  const player1 = new Player("p1")
  const game = Game.create(uuidv4())
  game.addPlayer(player1)

  expect(() => {
    game.start(player1.id)
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
