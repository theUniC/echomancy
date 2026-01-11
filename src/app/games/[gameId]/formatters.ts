import type { GameSteps } from "@/echomancy/domainmodel/game/Steps"

/**
 * Formats a game step name from SCREAMING_SNAKE_CASE to "Title Case".
 * Special cases:
 * - FIRST_MAIN -> "Main Phase 1"
 * - SECOND_MAIN -> "Main Phase 2"
 *
 * @param step - The game step to format
 * @returns Human-readable step name
 */
export function formatStepName(step: GameSteps): string {
  // Handle special cases for main phases
  if (step === "FIRST_MAIN") {
    return "Main Phase 1"
  }
  if (step === "SECOND_MAIN") {
    return "Main Phase 2"
  }

  // Convert SCREAMING_SNAKE_CASE to Title Case
  // Example: DECLARE_ATTACKERS -> Declare Attackers
  return step
    .split("_")
    .map((word) => word.charAt(0) + word.slice(1).toLowerCase())
    .join(" ")
}

/**
 * Formats the current phase and step for display.
 * Rules:
 * - Main phases: show formatted step name only
 * - Combat: show "Combat - {step name}"
 * - Beginning/Ending: show phase name or step name as appropriate
 *
 * @param phase - The current phase name (from GameSnapshot)
 * @param step - The current game step
 * @returns Formatted phase/step string for display
 */
export function formatPhaseAndStep(phase: string, step: GameSteps): string {
  // Main phases: show just the step name (already formatted nicely)
  if (step === "FIRST_MAIN" || step === "SECOND_MAIN") {
    return formatStepName(step)
  }

  // Combat: show "Combat - {step}"
  if (phase === "Combat") {
    return `Combat - ${formatStepName(step)}`
  }

  // Beginning phase: show step names individually
  if (phase === "Beginning") {
    return formatStepName(step)
  }

  // Ending phase: show step names individually
  if (phase === "Ending") {
    return formatStepName(step)
  }

  // Fallback: show phase name
  return phase
}
