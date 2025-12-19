import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import type { Effect } from "../../effects/Effect"
import type { EffectContext } from "../../effects/EffectContext"
import type { Game } from "../Game"
import { Step } from "../Steps"
import { addSpellToHand, advanceToStep, createStartedGame } from "./helpers"

/**
 * Dummy test effect that inserts extra phases:
 * - COMBAT (all combat steps)
 * - FIRST_MAIN
 *
 * This effect uses the scheduledSteps API to insert phases.
 */
class ExtraMainAndCombatEffect implements Effect {
  resolve(game: Game, _context: EffectContext): void {
    // Insert extra phases: COMBAT steps + FIRST_MAIN
    game.addScheduledSteps([
      Step.BEGINNING_OF_COMBAT,
      Step.DECLARE_ATTACKERS,
      Step.DECLARE_BLOCKERS,
      Step.COMBAT_DAMAGE,
      Step.END_OF_COMBAT,
      Step.FIRST_MAIN,
    ])
  }
}

describe("Structural Effects - Extra Phases MVP", () => {
  test("it inserts extra combat and main phases after resolving", () => {
    const { game, player1, player2 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    const spellCard: CardInstance = {
      instanceId: "spell-extra-phases",
      definition: {
        id: "spell-extra-phases",
        name: "Spell Extra Phases",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    addSpellToHand(game, player1.id, spellCard)

    // Cast the spell
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })

    // Spell is on stack, step hasn't changed
    expect(game.currentStep).toBe(Step.FIRST_MAIN)
    expect(game.getStack()).toHaveLength(1)

    // Both players pass priority to resolve the stack
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    // After resolution, we should still be in FIRST_MAIN
    // (extra phases inserted but not advanced yet)
    expect(game.getStack()).toHaveLength(0)
    expect(game.currentStep).toBe(Step.FIRST_MAIN)

    // Now advance step - should go to COMBAT (first extra phase)
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    expect(game.currentStep).toBe(Step.BEGINNING_OF_COMBAT)
  })

  test("it executes extra phases in the declared order", () => {
    const { game, player1, player2 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    const spellCard: CardInstance = {
      instanceId: "spell-phase-order",
      definition: {
        id: "spell-phase-order",
        name: "Spell Phase Order",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    addSpellToHand(game, player1.id, spellCard)

    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })

    // Resolve the stack
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    // Advance to first extra phase: COMBAT
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    expect(game.currentStep).toBe(Step.BEGINNING_OF_COMBAT)

    // Advance through COMBAT steps to next step
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.DECLARE_ATTACKERS)

    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.COMBAT_DAMAGE)

    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.END_OF_COMBAT)

    // After COMBAT, should go to second extra phase: FIRST_MAIN
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    expect(game.currentStep).toBe(Step.FIRST_MAIN)

    // After extra FIRST_MAIN, should resume normal flow (SECOND_MAIN)
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    expect(game.currentStep).toBe(Step.SECOND_MAIN)
  })

  test("it preserves priority behavior during extra phases", () => {
    const { game, player1, player2 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    const spellCard: CardInstance = {
      instanceId: "spell-priority-test",
      definition: {
        id: "spell-priority-test",
        name: "Spell Priority Test",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    addSpellToHand(game, player1.id, spellCard)

    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })

    // Resolve the stack
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    // Advance to extra COMBAT phase
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    expect(game.currentStep).toBe(Step.BEGINNING_OF_COMBAT)

    // Current player should have allowed actions in extra phase
    const allowedActions = game.getAllowedActionsFor(player1.id)
    expect(allowedActions.length).toBeGreaterThan(0)
    expect(allowedActions.includes("ADVANCE_STEP")).toBe(true)

    // Non-current player should have no actions in extra phase
    const opponentActions = game.getAllowedActionsFor(player2.id)
    expect(opponentActions.length).toBe(0)
  })

  test("it chains multiple extra phase effects correctly", () => {
    const { game, player1, player2 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    // First spell with extra phases
    const spellCard1: CardInstance = {
      instanceId: "spell-chain-1",
      definition: {
        id: "spell-chain-1",
        name: "Spell Chain 1",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    // Second spell with extra phases
    const spellCard2: CardInstance = {
      instanceId: "spell-chain-2",
      definition: {
        id: "spell-chain-2",
        name: "Spell Chain 2",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    addSpellToHand(game, player1.id, spellCard1)
    addSpellToHand(game, player2.id, spellCard2)

    // Cast first spell
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard1.instanceId,
      targets: [],
    })

    // Priority passes to player2, who casts in response
    game.apply({
      type: "CAST_SPELL",
      playerId: player2.id,
      cardId: spellCard2.instanceId,
      targets: [],
    })

    expect(game.getStack()).toHaveLength(2)

    // Both players pass to resolve spell2 (top of stack)
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    // spell2 resolved, spell1 still on stack
    expect(game.getStack()).toHaveLength(1)

    // Both players pass again to resolve spell1
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    // Both spells resolved, both inserted extra phases
    expect(game.getStack()).toHaveLength(0)

    // Both effects should have scheduled their phases
    // The exact order depends on implementation, but phases should execute
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    // Should enter one of the extra COMBAT phases
    expect(game.currentStep).toBe(Step.BEGINNING_OF_COMBAT)
  })

  test("it resumes normal turn flow after extra phases", () => {
    const { game, player1, player2 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    const spellCard: CardInstance = {
      instanceId: "spell-resume-flow",
      definition: {
        id: "spell-resume-flow",
        name: "Spell Resume Flow",
        type: "SPELL",
        effect: new ExtraMainAndCombatEffect(),
      },
      ownerId: player1.id,
    }

    addSpellToHand(game, player1.id, spellCard)

    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: spellCard.instanceId,
      targets: [],
    })

    // Resolve the stack
    game.apply({
      type: "PASS_PRIORITY",
      playerId: player2.id,
    })

    game.apply({
      type: "PASS_PRIORITY",
      playerId: player1.id,
    })

    // Advance through all extra phases
    // From FIRST_MAIN -> COMBAT steps -> extra FIRST_MAIN -> normal flow

    // Enter extra COMBAT
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.BEGINNING_OF_COMBAT)

    // Complete COMBAT steps
    advanceToStep(game, Step.END_OF_COMBAT)

    // Enter extra FIRST_MAIN
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.FIRST_MAIN)

    // After extra FIRST_MAIN, continue to SECOND_MAIN (normal flow)
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.SECOND_MAIN)

    // Continue normal turn sequence
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.END_STEP)

    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })
    expect(game.currentStep).toBe(Step.CLEANUP)

    // Turn should end normally
    const currentPlayer = game.currentPlayerId
    game.apply({
      type: "ADVANCE_STEP",
      playerId: player1.id,
    })

    // Should advance to next player's turn
    expect(game.currentPlayerId).not.toBe(currentPlayer)
    expect(game.currentStep).toBe(Step.UNTAP)
  })
})
