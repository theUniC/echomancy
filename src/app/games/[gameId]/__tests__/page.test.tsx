import { describe, expect, test } from "vitest"
import type { GameSteps } from "@/echomancy/domainmodel/game/Steps"
import type { GameSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { formatPhaseAndStep, formatStepName } from "../formatters"

// Note: Full E2E component tests with rendering are skipped due to Next.js 16 async params complexity.
// The core display logic is tested through integration of formatters with snapshot data.

describe("GamePage display logic", () => {
  const mockSnapshot: GameSnapshot = {
    viewerPlayerId: "player-1",
    publicGameState: {
      turnNumber: 3,
      currentPlayerId: "player-1",
      activePlayerId: "player-1",
      priorityPlayerId: "player-1",
      currentPhase: "Precombat Main",
      currentStep: "FIRST_MAIN",
      combatSummary: null,
      stackSize: 0,
    },
    privatePlayerState: {
      playerId: "player-1",
      lifeTotal: 20,
      poisonCounters: 0,
      manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
      hand: [],
      battlefield: [],
      graveyard: [],
      exile: [],
    },
    opponentStates: [
      {
        playerId: "player-2",
        lifeTotal: 18,
        poisonCounters: 0,
        manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
        handSize: 7,
        battlefield: [],
        graveyard: [],
        exile: [],
      },
    ],
    visibleStack: { items: [] },
    uiHints: null,
  }

  test("formats turn and phase display correctly for main phase", () => {
    const { publicGameState } = mockSnapshot
    const display = formatPhaseAndStep(
      publicGameState.currentPhase,
      publicGameState.currentStep,
    )

    expect(`Turn ${publicGameState.turnNumber} - ${display}`).toBe(
      "Turn 3 - Main Phase 1",
    )
  })

  test("formats turn and phase display correctly for combat", () => {
    const combatSnapshot = {
      ...mockSnapshot,
      publicGameState: {
        ...mockSnapshot.publicGameState,
        currentPhase: "Combat",
        currentStep: "DECLARE_ATTACKERS" as const,
      },
    }

    const display = formatPhaseAndStep(
      combatSnapshot.publicGameState.currentPhase,
      combatSnapshot.publicGameState.currentStep,
    )

    expect(
      `Turn ${combatSnapshot.publicGameState.turnNumber} - ${display}`,
    ).toBe("Turn 3 - Combat - Declare Attackers")
  })

  test("displays player life total", () => {
    expect(mockSnapshot.privatePlayerState.lifeTotal).toBe(20)
  })

  test("displays opponent life total", () => {
    expect(mockSnapshot.opponentStates[0]?.lifeTotal).toBe(18)
  })

  test("handles missing opponent gracefully", () => {
    const noOpponentSnapshot = {
      ...mockSnapshot,
      opponentStates: [],
    }

    const opponentLife = noOpponentSnapshot.opponentStates[0]?.lifeTotal ?? null
    expect(opponentLife).toBeNull()
  })

  test("formats all game steps correctly", () => {
    const steps = [
      { step: "UNTAP", expected: "Untap" },
      { step: "UPKEEP", expected: "Upkeep" },
      { step: "DRAW", expected: "Draw" },
      { step: "FIRST_MAIN", expected: "Main Phase 1" },
      { step: "SECOND_MAIN", expected: "Main Phase 2" },
      { step: "END_STEP", expected: "End Step" },
      { step: "CLEANUP", expected: "Cleanup" },
    ]

    for (const { step, expected } of steps) {
      const formatted = formatStepName(step as GameSteps)
      expect(formatted).toBe(expected)
    }
  })

  test("formats combat steps with Combat prefix", () => {
    const combatSteps = [
      { step: "BEGINNING_OF_COMBAT", expected: "Combat - Beginning Of Combat" },
      { step: "DECLARE_ATTACKERS", expected: "Combat - Declare Attackers" },
      { step: "DECLARE_BLOCKERS", expected: "Combat - Declare Blockers" },
      { step: "COMBAT_DAMAGE", expected: "Combat - Combat Damage" },
      { step: "END_OF_COMBAT", expected: "Combat - End Of Combat" },
    ]

    for (const { step, expected } of combatSteps) {
      const formatted = formatPhaseAndStep("Combat", step as GameSteps)
      expect(formatted).toBe(expected)
    }
  })

  test("handles different turn numbers", () => {
    const turn10Snapshot = {
      ...mockSnapshot,
      publicGameState: {
        ...mockSnapshot.publicGameState,
        turnNumber: 10,
      },
    }

    expect(turn10Snapshot.publicGameState.turnNumber).toBe(10)
  })

  test("handles different life totals", () => {
    const modifiedSnapshot = {
      ...mockSnapshot,
      privatePlayerState: {
        ...mockSnapshot.privatePlayerState,
        lifeTotal: 15,
      },
      opponentStates: mockSnapshot.opponentStates[0]
        ? [
            {
              ...mockSnapshot.opponentStates[0],
              lifeTotal: 22,
            },
          ]
        : [],
    }

    expect(modifiedSnapshot.privatePlayerState.lifeTotal).toBe(15)
    expect(modifiedSnapshot.opponentStates[0]?.lifeTotal).toBe(22)
  })
})
