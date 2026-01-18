import { describe, expect, test } from "bun:test"
import { createStartedGame } from "../../__tests__/helpers"
import { HasPriority } from "../HasPriority"

describe("HasPriority Specification", () => {
  test("returns true when player has priority", () => {
    const { game, player1 } = createStartedGame()
    const spec = new HasPriority()

    // player1 starts with priority
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })

  test("returns false when player does not have priority", () => {
    const { game, player2 } = createStartedGame()
    const spec = new HasPriority()

    // player2 does not have priority at start
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(false)
  })
})
