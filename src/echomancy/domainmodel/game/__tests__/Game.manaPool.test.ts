import { expect, test } from "vitest"
import {
  InsufficientManaError,
  InvalidManaAmountError,
  PlayerNotFoundError,
} from "../GameErrors"
import { Step } from "../Steps"
import { advanceToStep, createStartedGame } from "./helpers"

test("player mana pools start empty", () => {
  const { game, player1, player2 } = createStartedGame()

  const pool1 = game.getManaPool(player1.id)
  const pool2 = game.getManaPool(player2.id)

  expect(pool1.W).toBe(0)
  expect(pool1.U).toBe(0)
  expect(pool1.B).toBe(0)
  expect(pool1.R).toBe(0)
  expect(pool1.G).toBe(0)
  expect(pool1.C).toBe(0)

  expect(pool2.W).toBe(0)
  expect(pool2.U).toBe(0)
  expect(pool2.B).toBe(0)
  expect(pool2.R).toBe(0)
  expect(pool2.G).toBe(0)
  expect(pool2.C).toBe(0)
})

test("addMana increases pool for that color", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(1)
  expect(pool.W).toBe(0)
  expect(pool.U).toBe(0)
  expect(pool.B).toBe(0)
  expect(pool.R).toBe(0)
  expect(pool.C).toBe(0)
})

test("addMana accumulates", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)
  game.addMana(player1.id, "G", 2)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(3)
})

test("addMana works for all colors", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "W", 1)
  game.addMana(player1.id, "U", 2)
  game.addMana(player1.id, "B", 3)
  game.addMana(player1.id, "R", 4)
  game.addMana(player1.id, "G", 5)
  game.addMana(player1.id, "C", 6)

  const pool = game.getManaPool(player1.id)
  expect(pool.W).toBe(1)
  expect(pool.U).toBe(2)
  expect(pool.B).toBe(3)
  expect(pool.R).toBe(4)
  expect(pool.G).toBe(5)
  expect(pool.C).toBe(6)
})

test("addMana throws InvalidManaAmountError if amount is 0", () => {
  const { game, player1 } = createStartedGame()

  expect(() => game.addMana(player1.id, "G", 0)).toThrow(InvalidManaAmountError)
})

test("addMana throws InvalidManaAmountError if amount is negative", () => {
  const { game, player1 } = createStartedGame()

  expect(() => game.addMana(player1.id, "G", -1)).toThrow(
    InvalidManaAmountError,
  )
})

test("addMana throws PlayerNotFoundError if player doesn't exist", () => {
  const { game } = createStartedGame()

  expect(() => game.addMana("nonexistent", "G", 1)).toThrow(PlayerNotFoundError)
})

test("spendMana decreases pool for that color", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 3)
  game.spendMana(player1.id, "G", 2)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(1)
})

test("spendMana throws InsufficientManaError if insufficient mana", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)

  expect(() => game.spendMana(player1.id, "G", 2)).toThrow(
    InsufficientManaError,
  )
})

test("spendMana throws InsufficientManaError if no mana at all", () => {
  const { game, player1 } = createStartedGame()

  expect(() => game.spendMana(player1.id, "G", 1)).toThrow(
    InsufficientManaError,
  )
})

test("spendMana throws InvalidManaAmountError if amount is 0", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 5)

  expect(() => game.spendMana(player1.id, "G", 0)).toThrow(
    InvalidManaAmountError,
  )
})

test("spendMana throws InvalidManaAmountError if amount is negative", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 5)

  expect(() => game.spendMana(player1.id, "G", -1)).toThrow(
    InvalidManaAmountError,
  )
})

test("spendMana throws PlayerNotFoundError if player doesn't exist", () => {
  const { game } = createStartedGame()

  expect(() => game.spendMana("nonexistent", "G", 1)).toThrow(
    PlayerNotFoundError,
  )
})

test("mana pools are isolated per player", () => {
  const { game, player1, player2 } = createStartedGame()

  game.addMana(player1.id, "G", 2)

  const pool1 = game.getManaPool(player1.id)
  const pool2 = game.getManaPool(player2.id)

  expect(pool1.G).toBe(2)
  expect(pool2.G).toBe(0)
})

test("getManaPool returns a snapshot (mutations don't affect game state)", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 2)

  const pool1 = game.getManaPool(player1.id)
  // Mutate the returned pool
  pool1.G = 999

  // Get a fresh snapshot
  const pool2 = game.getManaPool(player1.id)
  expect(pool2.G).toBe(2)
})

test("getManaPool throws PlayerNotFoundError if player doesn't exist", () => {
  const { game } = createStartedGame()

  expect(() => game.getManaPool("nonexistent")).toThrow(PlayerNotFoundError)
})

test("mana pool clears on entering CLEANUP", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 2)
  game.addMana(player1.id, "R", 3)

  advanceToStep(game, Step.CLEANUP)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(0)
  expect(pool.R).toBe(0)
})

test("mana pool does not clear on normal step changes before CLEANUP (MVP behavior)", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 2)

  // Advance to SECOND_MAIN (after FIRST_MAIN)
  advanceToStep(game, Step.SECOND_MAIN)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(2)
})

test("mana pool does not clear on entering END_STEP (MVP behavior)", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 2)

  advanceToStep(game, Step.END_STEP)

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(2)
})

test("clearManaPool clears all colors for a player", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "W", 1)
  game.addMana(player1.id, "U", 2)
  game.addMana(player1.id, "B", 3)
  game.addMana(player1.id, "R", 4)
  game.addMana(player1.id, "G", 5)
  game.addMana(player1.id, "C", 6)

  game.clearManaPool(player1.id)

  const pool = game.getManaPool(player1.id)
  expect(pool.W).toBe(0)
  expect(pool.U).toBe(0)
  expect(pool.B).toBe(0)
  expect(pool.R).toBe(0)
  expect(pool.G).toBe(0)
  expect(pool.C).toBe(0)
})

test("clearManaPool only clears the specified player's pool", () => {
  const { game, player1, player2 } = createStartedGame()

  game.addMana(player1.id, "G", 2)
  game.addMana(player2.id, "R", 3)

  game.clearManaPool(player1.id)

  const pool1 = game.getManaPool(player1.id)
  const pool2 = game.getManaPool(player2.id)

  expect(pool1.G).toBe(0)
  expect(pool2.R).toBe(3)
})

test("clearManaPool throws PlayerNotFoundError if player doesn't exist", () => {
  const { game } = createStartedGame()

  expect(() => game.clearManaPool("nonexistent")).toThrow(PlayerNotFoundError)
})

test("clearAllManaPools clears all players' pools", () => {
  const { game, player1, player2 } = createStartedGame()

  game.addMana(player1.id, "G", 2)
  game.addMana(player2.id, "R", 3)

  game.clearAllManaPools()

  const pool1 = game.getManaPool(player1.id)
  const pool2 = game.getManaPool(player2.id)

  expect(pool1.G).toBe(0)
  expect(pool2.R).toBe(0)
})
