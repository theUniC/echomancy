import { describe, expect, test } from "vitest"
import type { GameSteps } from "@/echomancy/domainmodel/game/Steps"
import { formatPhaseAndStep, formatStepName } from "../formatters"

describe("formatStepName", () => {
  test("formats UNTAP to Title Case", () => {
    expect(formatStepName("UNTAP")).toBe("Untap")
  })

  test("formats UPKEEP to Title Case", () => {
    expect(formatStepName("UPKEEP")).toBe("Upkeep")
  })

  test("formats DRAW to Title Case", () => {
    expect(formatStepName("DRAW")).toBe("Draw")
  })

  test("formats FIRST_MAIN to Main Phase 1", () => {
    expect(formatStepName("FIRST_MAIN")).toBe("Main Phase 1")
  })

  test("formats BEGINNING_OF_COMBAT to Title Case", () => {
    expect(formatStepName("BEGINNING_OF_COMBAT")).toBe("Beginning Of Combat")
  })

  test("formats DECLARE_ATTACKERS to Title Case", () => {
    expect(formatStepName("DECLARE_ATTACKERS")).toBe("Declare Attackers")
  })

  test("formats DECLARE_BLOCKERS to Title Case", () => {
    expect(formatStepName("DECLARE_BLOCKERS")).toBe("Declare Blockers")
  })

  test("formats COMBAT_DAMAGE to Title Case", () => {
    expect(formatStepName("COMBAT_DAMAGE")).toBe("Combat Damage")
  })

  test("formats END_OF_COMBAT to Title Case", () => {
    expect(formatStepName("END_OF_COMBAT")).toBe("End Of Combat")
  })

  test("formats SECOND_MAIN to Main Phase 2", () => {
    expect(formatStepName("SECOND_MAIN")).toBe("Main Phase 2")
  })

  test("formats END_STEP to Title Case", () => {
    expect(formatStepName("END_STEP")).toBe("End Step")
  })

  test("formats CLEANUP to Title Case", () => {
    expect(formatStepName("CLEANUP")).toBe("Cleanup")
  })
})

describe("formatPhaseAndStep", () => {
  describe("main phases", () => {
    test("formats first main phase", () => {
      expect(formatPhaseAndStep("Precombat Main", "FIRST_MAIN")).toBe(
        "Main Phase 1",
      )
    })

    test("formats second main phase", () => {
      expect(formatPhaseAndStep("Postcombat Main", "SECOND_MAIN")).toBe(
        "Main Phase 2",
      )
    })
  })

  describe("combat phase", () => {
    test("formats beginning of combat", () => {
      expect(formatPhaseAndStep("Combat", "BEGINNING_OF_COMBAT")).toBe(
        "Combat - Beginning Of Combat",
      )
    })

    test("formats declare attackers", () => {
      expect(formatPhaseAndStep("Combat", "DECLARE_ATTACKERS")).toBe(
        "Combat - Declare Attackers",
      )
    })

    test("formats declare blockers", () => {
      expect(formatPhaseAndStep("Combat", "DECLARE_BLOCKERS")).toBe(
        "Combat - Declare Blockers",
      )
    })

    test("formats combat damage", () => {
      expect(formatPhaseAndStep("Combat", "COMBAT_DAMAGE")).toBe(
        "Combat - Combat Damage",
      )
    })

    test("formats end of combat", () => {
      expect(formatPhaseAndStep("Combat", "END_OF_COMBAT")).toBe(
        "Combat - End Of Combat",
      )
    })
  })

  describe("beginning phase", () => {
    test("formats untap step", () => {
      expect(formatPhaseAndStep("Beginning", "UNTAP")).toBe("Untap")
    })

    test("formats upkeep step", () => {
      expect(formatPhaseAndStep("Beginning", "UPKEEP")).toBe("Upkeep")
    })

    test("formats draw step", () => {
      expect(formatPhaseAndStep("Beginning", "DRAW")).toBe("Draw")
    })
  })

  describe("ending phase", () => {
    test("formats end step", () => {
      expect(formatPhaseAndStep("Ending", "END_STEP")).toBe("End Step")
    })

    test("formats cleanup step", () => {
      expect(formatPhaseAndStep("Ending", "CLEANUP")).toBe("Cleanup")
    })
  })

  describe("edge cases", () => {
    test("handles unknown phase by returning phase name", () => {
      expect(formatPhaseAndStep("Unknown", "UNTAP" as GameSteps)).toBe(
        "Unknown",
      )
    })
  })
})
