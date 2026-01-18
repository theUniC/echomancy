import { describe, expect, test } from "vitest"
import {
  addCreatureToBattlefield,
  createStartedGame,
  createTestCreature,
} from "../../__tests__/helpers"
import { Step } from "../../Steps"
import { CanDeclareAttacker } from "../CanDeclareAttacker"

describe("CanDeclareAttacker Specification", () => {
  test("returns false when not in DECLARE_ATTACKERS step", () => {
    const { game, player1 } = createStartedGame()
    const spec = new CanDeclareAttacker()

    // UNTAP step
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })

  test("returns false when player is not current player", () => {
    const { game, player2 } = createStartedGame()
    const creature = createTestCreature(player2.id)
    addCreatureToBattlefield(game, player2.id, creature)

    // Manually set step to DECLARE_ATTACKERS
    while (game.currentStep !== Step.DECLARE_ATTACKERS) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }

    const spec = new CanDeclareAttacker()
    // player2 is not current player
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(false)
  })

  test("returns false when player has no creatures", () => {
    const { game, player1 } = createStartedGame()

    // Advance to DECLARE_ATTACKERS
    while (game.currentStep !== Step.DECLARE_ATTACKERS) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }

    const spec = new CanDeclareAttacker()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })
})
