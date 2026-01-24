import { describe, expect, test } from "vitest"
import {
  NotMainPhaseError,
  NotYourTurnError,
  StackNotEmptyError,
} from "../GameErrors"

describe("Timing Errors", () => {
  describe("NotYourTurnError", () => {
    test("has clear message", () => {
      const error = new NotYourTurnError()

      expect(error.message).toContain("your turn")
      expect(error.message).toContain("main phase")
      expect(error.message).toContain("empty")
    })

    test("extends GameError", () => {
      const error = new NotYourTurnError()

      expect(error.name).toBe("NotYourTurnError")
    })
  })

  describe("NotMainPhaseError", () => {
    test("has clear message", () => {
      const error = new NotMainPhaseError()

      expect(error.message).toContain("main phase")
    })

    test("extends GameError", () => {
      const error = new NotMainPhaseError()

      expect(error.name).toBe("NotMainPhaseError")
    })
  })

  describe("StackNotEmptyError", () => {
    test("has clear message", () => {
      const error = new StackNotEmptyError()

      expect(error.message).toContain("stack")
      expect(error.message).toContain("empty")
    })

    test("extends GameError", () => {
      const error = new StackNotEmptyError()

      expect(error.name).toBe("StackNotEmptyError")
    })
  })

  describe("Error messages for creatures", () => {
    test("NotYourTurnError mentions Flash for creatures", () => {
      const error = new NotYourTurnError(true)

      expect(error.message).toContain("Flash")
    })

    test("NotMainPhaseError mentions Flash for creatures", () => {
      const error = new NotMainPhaseError(true)

      expect(error.message).toContain("Flash")
    })

    test("StackNotEmptyError mentions Flash for creatures", () => {
      const error = new StackNotEmptyError(true)

      expect(error.message).toContain("Flash")
    })
  })
})
