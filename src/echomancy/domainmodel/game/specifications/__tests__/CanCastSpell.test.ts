import { describe, expect, test } from "vitest"
import {
  addSpellToHand,
  createCreatureWithFlash,
  createGameInMainPhase,
  createStartedGame,
  createTestCreature,
  createTestSpell,
} from "../../__tests__/helpers"
import { Step } from "../../Steps"
import { CanCastSpell } from "../CanCastSpell"

describe("CanCastSpell Specification", () => {
  test("returns true when player has spell in hand during main phase", () => {
    const { game, player1 } = createGameInMainPhase()
    const spell = createTestSpell(player1.id)
    addSpellToHand(game, player1.id, spell)

    const spec = new CanCastSpell()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })

  test("returns false when not in main phase with only sorcery-speed spells", () => {
    const { game, player1 } = createStartedGame()
    // Create a sorcery (not an instant)
    const sorcery = {
      instanceId: "test-sorcery",
      definition: {
        id: "test-sorcery",
        name: "Test Sorcery",
        types: ["SORCERY" as const],
      },
      ownerId: player1.id,
    }
    addSpellToHand(game, player1.id, sorcery)

    const spec = new CanCastSpell()
    // UNTAP phase, not main phase - sorcery cannot be cast
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

  test("returns true for instant-speed spell on opponent turn", () => {
    const { game, player1, player2 } = createStartedGame()

    // Advance to player1's main phase
    while (game.currentStep !== Step.FIRST_MAIN) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }
    expect(game.currentPlayerId).toBe(player1.id)

    // Player2 has an instant in hand
    const instant = createTestSpell(player2.id)
    addSpellToHand(game, player2.id, instant)

    const spec = new CanCastSpell()
    // Player2 can cast instant even on player1's turn
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(true)
  })

  test("returns true for Flash creature on opponent turn", () => {
    const { game, player1, player2 } = createStartedGame()

    // Advance to player1's main phase
    while (game.currentStep !== Step.FIRST_MAIN) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }
    expect(game.currentPlayerId).toBe(player1.id)

    // Player2 has a Flash creature in hand
    const flashCreature = createCreatureWithFlash(player2.id)
    const playerState = game.getPlayerState(player2.id)
    playerState.hand.cards.push(flashCreature)

    const spec = new CanCastSpell()
    // Player2 can cast Flash creature even on player1's turn
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(true)
  })

  test("returns false for non-Flash creature on opponent turn", () => {
    const { game, player1, player2 } = createStartedGame()

    // Advance to player1's main phase
    while (game.currentStep !== Step.FIRST_MAIN) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }
    expect(game.currentPlayerId).toBe(player1.id)

    // Player2 has a regular creature in hand
    const creature = createTestCreature(player2.id)
    const playerState = game.getPlayerState(player2.id)
    playerState.hand.cards.push(creature)

    const spec = new CanCastSpell()
    // Player2 cannot cast regular creature on player1's turn
    expect(spec.isSatisfiedBy({ game, playerId: player2.id })).toBe(false)
  })

  test("returns true for instant-speed spell during combat", () => {
    const { game, player1 } = createStartedGame()

    // Advance to combat phase
    while (game.currentStep !== Step.BEGINNING_OF_COMBAT) {
      game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
    }

    // Player1 has an instant in hand
    const instant = createTestSpell(player1.id)
    addSpellToHand(game, player1.id, instant)

    const spec = new CanCastSpell()
    // Player can cast instant during combat
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })
})
