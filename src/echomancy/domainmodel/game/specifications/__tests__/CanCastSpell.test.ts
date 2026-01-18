import { describe, expect, test } from "bun:test"
import {
  addSpellToHand,
  createGameInMainPhase,
  createStartedGame,
  createTestSpell,
} from "../../__tests__/helpers"
import { CanCastSpell } from "../CanCastSpell"

describe("CanCastSpell Specification", () => {
  test("returns true when player has spell in hand during main phase", () => {
    const { game, player1 } = createGameInMainPhase()
    const spell = createTestSpell(player1.id)
    addSpellToHand(game, player1.id, spell)

    const spec = new CanCastSpell()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })

  test("returns false when not in main phase", () => {
    const { game, player1 } = createStartedGame()
    const spell = createTestSpell(player1.id)
    addSpellToHand(game, player1.id, spell)

    const spec = new CanCastSpell()
    // UNTAP phase, not main phase
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })

  test("returns false when player has no spells in hand", () => {
    const { game, player1 } = createGameInMainPhase()

    const spec = new CanCastSpell()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })

  test("returns true for any player with spell in hand during main phase", () => {
    const { game, player2 } = createGameInMainPhase()
    const spell = createTestSpell(player2.id)
    addSpellToHand(game, player2.id, spell)

    const spec = new CanCastSpell()
    // canCastSpell doesn't check current player - that's done by hasPriority check in getAllowedActionsFor
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(true)
  })
})
