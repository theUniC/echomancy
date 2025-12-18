import { type GameSteps, Step } from "./Steps"

const STEP_ORDER: GameSteps[] = [
  Step.UNTAP,
  Step.UPKEEP,
  Step.DRAW,
  Step.MAIN1,
  Step.BEGINNING_OF_COMBAT,
  Step.DECLARE_ATTACKERS,
  Step.DECLARE_BLOCKERS,
  Step.COMBAT_DAMAGE,
  Step.END_OF_COMBAT,
  Step.SECOND_MAIN,
  Step.END_STEP,
  Step.CLEANUP,
]

export type StepResult = {
  nextStep: GameSteps
  shouldAdvancePlayer: boolean
}

export function advance(currentStep: GameSteps): StepResult {
  const currentStepIndex = STEP_ORDER.indexOf(currentStep)
  const nextStep = STEP_ORDER[(currentStepIndex + 1) % STEP_ORDER.length]

  return {
    nextStep,
    shouldAdvancePlayer: nextStep === Step.UNTAP,
  }
}
