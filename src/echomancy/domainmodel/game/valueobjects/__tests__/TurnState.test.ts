import { describe, expect, test } from "vitest"
import { Step } from "../../Steps"
import { TurnState } from "../TurnState"

describe("TurnState Value Object", () => {
  describe("initial", () => {
    test("creates initial state with starting player", () => {
      const state = TurnState.initial("player-1")

      expect(state.currentPlayerId).toBe("player-1")
      expect(state.currentStep).toBe(Step.UNTAP)
      expect(state.turnNumber).toBe(1)
      expect(state.playedLands).toBe(0)
    })
  })

  describe("fromSnapshot", () => {
    test("creates state from snapshot", () => {
      const snapshot = {
        currentPlayerId: "player-2",
        currentStep: Step.FIRST_MAIN,
        turnNumber: 3,
        playedLands: 1,
      }

      const state = TurnState.fromSnapshot(snapshot)

      expect(state.currentPlayerId).toBe("player-2")
      expect(state.currentStep).toBe(Step.FIRST_MAIN)
      expect(state.turnNumber).toBe(3)
      expect(state.playedLands).toBe(1)
    })
  })

  describe("withStep", () => {
    test("returns new state with updated step", () => {
      const state = TurnState.initial("player-1")

      const newState = state.withStep(Step.FIRST_MAIN)

      expect(newState.currentStep).toBe(Step.FIRST_MAIN)
      expect(newState.currentPlayerId).toBe("player-1")
      expect(state.currentStep).toBe(Step.UNTAP) // Original unchanged
    })
  })

  describe("withCurrentPlayer", () => {
    test("returns new state with updated player", () => {
      const state = TurnState.initial("player-1")

      const newState = state.withCurrentPlayer("player-2")

      expect(newState.currentPlayerId).toBe("player-2")
      expect(state.currentPlayerId).toBe("player-1") // Original unchanged
    })
  })

  describe("withIncrementedTurnNumber", () => {
    test("returns new state with incremented turn number", () => {
      const state = TurnState.initial("player-1")

      const newState = state.withIncrementedTurnNumber()

      expect(newState.turnNumber).toBe(2)
      expect(state.turnNumber).toBe(1) // Original unchanged
    })
  })

  describe("withLandPlayed", () => {
    test("returns new state with incremented lands played", () => {
      const state = TurnState.initial("player-1")

      const newState = state.withLandPlayed()

      expect(newState.playedLands).toBe(1)
      expect(state.playedLands).toBe(0) // Original unchanged
    })

    test("can increment multiple times", () => {
      const state = TurnState.initial("player-1")

      const newState = state.withLandPlayed().withLandPlayed()

      expect(newState.playedLands).toBe(2)
    })
  })

  describe("withResetLands", () => {
    test("returns new state with zero lands played", () => {
      const state = TurnState.initial("player-1").withLandPlayed()

      const newState = state.withResetLands()

      expect(newState.playedLands).toBe(0)
      expect(state.playedLands).toBe(1) // Original unchanged
    })
  })

  describe("forNewTurn", () => {
    test("creates state for new turn with next player", () => {
      const state = TurnState.fromSnapshot({
        currentPlayerId: "player-1",
        currentStep: Step.CLEANUP,
        turnNumber: 2,
        playedLands: 1,
      })

      const newState = state.forNewTurn("player-2")

      expect(newState.currentPlayerId).toBe("player-2")
      expect(newState.currentStep).toBe(Step.UNTAP)
      expect(newState.turnNumber).toBe(2) // Turn number NOT incremented here
      expect(newState.playedLands).toBe(0)
    })
  })

  describe("isMainPhase", () => {
    test("returns true for FIRST_MAIN", () => {
      const state = TurnState.initial("player-1").withStep(Step.FIRST_MAIN)

      expect(state.isMainPhase()).toBe(true)
    })

    test("returns true for SECOND_MAIN", () => {
      const state = TurnState.initial("player-1").withStep(Step.SECOND_MAIN)

      expect(state.isMainPhase()).toBe(true)
    })

    test("returns false for other steps", () => {
      const state = TurnState.initial("player-1").withStep(Step.COMBAT_DAMAGE)

      expect(state.isMainPhase()).toBe(false)
    })
  })

  describe("hasPlayedLand", () => {
    test("returns false when no lands played", () => {
      const state = TurnState.initial("player-1")

      expect(state.hasPlayedLand()).toBe(false)
    })

    test("returns true when land has been played", () => {
      const state = TurnState.initial("player-1").withLandPlayed()

      expect(state.hasPlayedLand()).toBe(true)
    })
  })

  describe("toSnapshot", () => {
    test("returns snapshot with all properties", () => {
      const state = TurnState.fromSnapshot({
        currentPlayerId: "player-1",
        currentStep: Step.FIRST_MAIN,
        turnNumber: 5,
        playedLands: 1,
      })

      const snapshot = state.toSnapshot()

      expect(snapshot).toEqual({
        currentPlayerId: "player-1",
        currentStep: Step.FIRST_MAIN,
        turnNumber: 5,
        playedLands: 1,
      })
    })
  })

  describe("immutability", () => {
    test("all with* methods return new instances", () => {
      const state = TurnState.initial("player-1")

      const state2 = state.withStep(Step.FIRST_MAIN)
      const state3 = state.withCurrentPlayer("player-2")
      const state4 = state.withLandPlayed()
      const state5 = state.withIncrementedTurnNumber()
      const state6 = state.forNewTurn("player-2")

      expect(state2).not.toBe(state)
      expect(state3).not.toBe(state)
      expect(state4).not.toBe(state)
      expect(state5).not.toBe(state)
      expect(state6).not.toBe(state)
    })
  })
})
