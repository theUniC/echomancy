import { describe, expect, test } from "vitest"
import { InsufficientManaError, ManaPool } from "../valueobjects/ManaPool"

describe("ManaPool Value Object", () => {
  describe("creation", () => {
    test("empty() creates pool with all zeros", () => {
      const pool = ManaPool.empty()
      expect(pool.get("W")).toBe(0)
      expect(pool.get("U")).toBe(0)
      expect(pool.get("B")).toBe(0)
      expect(pool.get("R")).toBe(0)
      expect(pool.get("G")).toBe(0)
      expect(pool.get("C")).toBe(0)
    })

    test("fromSnapshot() creates pool from snapshot", () => {
      const snapshot = { W: 1, U: 2, B: 3, R: 4, G: 5, C: 6 }
      const pool = ManaPool.fromSnapshot(snapshot)
      expect(pool.get("W")).toBe(1)
      expect(pool.get("U")).toBe(2)
      expect(pool.get("B")).toBe(3)
      expect(pool.get("R")).toBe(4)
      expect(pool.get("G")).toBe(5)
      expect(pool.get("C")).toBe(6)
    })
  })

  describe("add()", () => {
    test("adds mana and returns new instance", () => {
      const pool1 = ManaPool.empty()
      const pool2 = pool1.add("R", 2)

      expect(pool1.get("R")).toBe(0) // Original unchanged
      expect(pool2.get("R")).toBe(2) // New instance has mana
    })

    test("can add to existing mana", () => {
      const pool = ManaPool.empty().add("R", 2).add("R", 3)
      expect(pool.get("R")).toBe(5)
    })

    test("throws on negative amount", () => {
      const pool = ManaPool.empty()
      expect(() => pool.add("R", -1)).toThrow("Cannot add negative mana")
    })
  })

  describe("spend()", () => {
    test("spends mana and returns new instance", () => {
      const pool1 = ManaPool.empty().add("R", 3)
      const pool2 = pool1.spend("R", 2)

      expect(pool1.get("R")).toBe(3) // Original unchanged
      expect(pool2.get("R")).toBe(1) // New instance has less
    })

    test("throws InsufficientManaError when not enough mana", () => {
      const pool = ManaPool.empty().add("R", 1)
      expect(() => pool.spend("R", 2)).toThrow(InsufficientManaError)
    })

    test("throws on negative amount", () => {
      const pool = ManaPool.empty().add("R", 3)
      expect(() => pool.spend("R", -1)).toThrow("Cannot spend negative mana")
    })
  })

  describe("clear()", () => {
    test("returns empty pool", () => {
      const pool = ManaPool.empty().add("R", 3).add("U", 2)
      const cleared = pool.clear()

      expect(pool.get("R")).toBe(3) // Original unchanged
      expect(cleared.isEmpty()).toBe(true)
    })
  })

  describe("isEmpty()", () => {
    test("returns true for empty pool", () => {
      expect(ManaPool.empty().isEmpty()).toBe(true)
    })

    test("returns false when pool has mana", () => {
      expect(ManaPool.empty().add("R", 1).isEmpty()).toBe(false)
    })
  })

  describe("equals()", () => {
    test("returns true for equal pools", () => {
      const pool1 = ManaPool.empty().add("R", 2).add("U", 1)
      const pool2 = ManaPool.empty().add("R", 2).add("U", 1)
      expect(pool1.equals(pool2)).toBe(true)
    })

    test("returns false for different pools", () => {
      const pool1 = ManaPool.empty().add("R", 2)
      const pool2 = ManaPool.empty().add("R", 3)
      expect(pool1.equals(pool2)).toBe(false)
    })
  })

  describe("toSnapshot()", () => {
    test("returns snapshot matching pool state", () => {
      const pool = ManaPool.empty().add("R", 2).add("G", 1)
      const snapshot = pool.toSnapshot()

      expect(snapshot).toEqual({ W: 0, U: 0, B: 0, R: 2, G: 1, C: 0 })
    })
  })

  describe("total()", () => {
    test("returns sum of all mana", () => {
      const pool = ManaPool.empty().add("R", 2).add("U", 3).add("G", 1)
      expect(pool.total()).toBe(6)
    })
  })
})
