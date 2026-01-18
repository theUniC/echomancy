import { describe, expect, test } from "vitest"
import {
  createGameInMainPhase,
  createStartedGame,
} from "../../__tests__/helpers"
import { CanPlayLand } from "../CanPlayLand"

describe("CanPlayLand Specification", () => {
  test("returns true in main phase when no land played", () => {
    const { game, player1 } = createGameInMainPhase()
    const spec = new CanPlayLand()

    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })

  test("returns false when not current player", () => {
    const { game, player2 } = createGameInMainPhase()
    const spec = new CanPlayLand()

    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(false)
  })

  test("returns false when not in main phase", () => {
    const { game, player1 } = createStartedGame()
    const spec = new CanPlayLand()

    // UNTAP phase, not main phase
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })
})
