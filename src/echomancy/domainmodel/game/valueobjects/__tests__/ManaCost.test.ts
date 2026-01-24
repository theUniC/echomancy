import { describe, expect, test } from "vitest"
import { type ManaCost, ManaCostParser } from "../ManaCost"

describe("ManaCostParser", () => {
  describe("parse()", () => {
    test("parses empty cost string", () => {
      const result = ManaCostParser.parse("")
      expect(result).toEqual({ generic: 0 })
    })

    test("parses all generic cost", () => {
      const result = ManaCostParser.parse("4")
      expect(result).toEqual({ generic: 4 })
    })

    test("parses single colored mana", () => {
      const result = ManaCostParser.parse("U")
      expect(result).toEqual({ generic: 0, U: 1 })
    })

    test("parses multiple of same colored mana", () => {
      const result = ManaCostParser.parse("UUU")
      expect(result).toEqual({ generic: 0, U: 3 })
    })

    test("parses generic plus single color", () => {
      const result = ManaCostParser.parse("2U")
      expect(result).toEqual({ generic: 2, U: 1 })
    })

    test("parses generic plus multiple colors", () => {
      const result = ManaCostParser.parse("2UU")
      expect(result).toEqual({ generic: 2, U: 2 })
    })

    test("parses multi-color cost", () => {
      const result = ManaCostParser.parse("1WU")
      expect(result).toEqual({ generic: 1, W: 1, U: 1 })
    })

    test("parses colorless mana (C)", () => {
      const result = ManaCostParser.parse("2C")
      expect(result).toEqual({ generic: 2, C: 1 })
    })

    test("parses all color combinations", () => {
      const result = ManaCostParser.parse("WUBRG")
      expect(result).toEqual({ generic: 0, W: 1, U: 1, B: 1, R: 1, G: 1 })
    })

    test("parses complex multi-digit generic cost", () => {
      const result = ManaCostParser.parse("12UU")
      expect(result).toEqual({ generic: 12, U: 2 })
    })

    test("handles cost with only colorless", () => {
      const result = ManaCostParser.parse("CCC")
      expect(result).toEqual({ generic: 0, C: 3 })
    })

    test("throws error for invalid characters", () => {
      expect(() => ManaCostParser.parse("2XX")).toThrow(
        "Invalid mana cost format: '2XX'",
      )
    })

    test("throws error for lowercase letters", () => {
      expect(() => ManaCostParser.parse("2uu")).toThrow(
        "Invalid mana cost format: '2uu'",
      )
    })
  })
})

describe("ManaCost type", () => {
  test("can be created with all properties", () => {
    const cost: ManaCost = {
      generic: 2,
      W: 1,
      U: 1,
      B: 0,
      R: 0,
      G: 0,
      C: 0,
    }
    expect(cost.generic).toBe(2)
    expect(cost.W).toBe(1)
    expect(cost.U).toBe(1)
  })

  test("can be created with only generic", () => {
    const cost: ManaCost = { generic: 4 }
    expect(cost.generic).toBe(4)
    expect(cost.W).toBeUndefined()
  })

  test("can be created with only colored", () => {
    const cost: ManaCost = { generic: 0, U: 2 }
    expect(cost.generic).toBe(0)
    expect(cost.U).toBe(2)
  })
})
