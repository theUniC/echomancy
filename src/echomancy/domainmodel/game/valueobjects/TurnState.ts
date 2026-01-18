/**
 * TurnState Value Object
 *
 * Immutable value object that holds turn-related game state.
 * Groups related turn properties for cleaner state management.
 *
 * Properties:
 * - currentPlayerId: The player whose turn it is
 * - currentStep: The current step/phase of the turn
 * - turnNumber: The current turn number (starts at 1)
 * - playedLands: Number of lands played this turn
 *
 * @example
 * const turnState = TurnState.initial("player-1")
 * const nextState = turnState.withStep(Step.FIRST_MAIN)
 */

import type { GameSteps } from "../Steps"
import { Step } from "../Steps"

export type TurnStateSnapshot = {
  currentPlayerId: string
  currentStep: GameSteps
  turnNumber: number
  playedLands: number
}

export class TurnState {
  readonly currentPlayerId: string
  readonly currentStep: GameSteps
  readonly turnNumber: number
  readonly playedLands: number

  private constructor(snapshot: TurnStateSnapshot) {
    this.currentPlayerId = snapshot.currentPlayerId
    this.currentStep = snapshot.currentStep
    this.turnNumber = snapshot.turnNumber
    this.playedLands = snapshot.playedLands
  }

  /**
   * Creates initial turn state for game start.
   */
  static initial(startingPlayerId: string): TurnState {
    return new TurnState({
      currentPlayerId: startingPlayerId,
      currentStep: Step.UNTAP,
      turnNumber: 1,
      playedLands: 0,
    })
  }

  /**
   * Creates TurnState from a snapshot.
   */
  static fromSnapshot(snapshot: TurnStateSnapshot): TurnState {
    return new TurnState(snapshot)
  }

  /**
   * Returns a new TurnState with the specified step.
   */
  withStep(step: GameSteps): TurnState {
    return new TurnState({
      ...this.toSnapshot(),
      currentStep: step,
    })
  }

  /**
   * Returns a new TurnState with the specified current player.
   */
  withCurrentPlayer(playerId: string): TurnState {
    return new TurnState({
      ...this.toSnapshot(),
      currentPlayerId: playerId,
    })
  }

  /**
   * Returns a new TurnState with incremented turn number.
   */
  withIncrementedTurnNumber(): TurnState {
    return new TurnState({
      ...this.toSnapshot(),
      turnNumber: this.turnNumber + 1,
    })
  }

  /**
   * Returns a new TurnState with incremented lands played.
   */
  withLandPlayed(): TurnState {
    return new TurnState({
      ...this.toSnapshot(),
      playedLands: this.playedLands + 1,
    })
  }

  /**
   * Returns a new TurnState with reset lands played (for new turn).
   */
  withResetLands(): TurnState {
    return new TurnState({
      ...this.toSnapshot(),
      playedLands: 0,
    })
  }

  /**
   * Returns a new TurnState for the start of a new turn.
   */
  forNewTurn(nextPlayerId: string): TurnState {
    return new TurnState({
      currentPlayerId: nextPlayerId,
      currentStep: Step.UNTAP,
      turnNumber: this.turnNumber,
      playedLands: 0,
    })
  }

  /**
   * Checks if the current step is a main phase.
   */
  isMainPhase(): boolean {
    return (
      this.currentStep === Step.FIRST_MAIN ||
      this.currentStep === Step.SECOND_MAIN
    )
  }

  /**
   * Checks if lands have been played this turn.
   */
  hasPlayedLand(): boolean {
    return this.playedLands > 0
  }

  /**
   * Returns a snapshot for persistence or comparison.
   */
  toSnapshot(): TurnStateSnapshot {
    return {
      currentPlayerId: this.currentPlayerId,
      currentStep: this.currentStep,
      turnNumber: this.turnNumber,
      playedLands: this.playedLands,
    }
  }
}
