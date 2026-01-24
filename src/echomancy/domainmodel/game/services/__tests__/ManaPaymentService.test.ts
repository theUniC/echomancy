import { describe, expect, test } from "vitest"
import { ManaPool } from "../../valueobjects/ManaPool"
import { ManaPaymentService } from "../ManaPaymentService"

describe("ManaPaymentService", () => {
  describe("canPayCost()", () => {
    test("can pay zero cost with empty pool", () => {
      const pool = ManaPool.empty().toSnapshot()
      const cost = { generic: 0 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("can pay all generic cost with any colored mana", () => {
      const pool = ManaPool.empty().add("R", 3).toSnapshot()
      const cost = { generic: 3 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("can pay all generic cost with colorless mana", () => {
      const pool = ManaPool.empty().add("C", 3).toSnapshot()
      const cost = { generic: 3 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("can pay colored cost with exact color", () => {
      const pool = ManaPool.empty().add("U", 2).toSnapshot()
      const cost = { generic: 0, U: 2 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("can pay mixed cost with exact colors", () => {
      const pool = ManaPool.empty().add("W", 1).add("U", 1).toSnapshot()
      const cost = { generic: 0, W: 1, U: 1 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("can pay generic plus colored with sufficient mana", () => {
      const pool = ManaPool.empty()
        .add("U", 2)
        .add("G", 1)
        .add("C", 1)
        .toSnapshot()
      const cost = { generic: 2, U: 2 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("cannot pay when insufficient colored mana", () => {
      const pool = ManaPool.empty().add("U", 1).toSnapshot()
      const cost = { generic: 0, U: 2 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(false)
    })

    test("cannot pay when wrong color", () => {
      const pool = ManaPool.empty().add("R", 2).toSnapshot()
      const cost = { generic: 0, U: 2 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(false)
    })

    test("cannot pay when insufficient total mana for generic", () => {
      const pool = ManaPool.empty().add("U", 2).add("G", 1).toSnapshot()
      const cost = { generic: 2, U: 2 }
      // Need U:2 + 2 generic, but only have U:2 + G:1 (1 short)
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(false)
    })

    test("can pay colorless requirement with C mana only", () => {
      const pool = ManaPool.empty().add("C", 1).toSnapshot()
      const cost = { generic: 0, C: 1 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })

    test("cannot pay colorless requirement with colored mana", () => {
      const pool = ManaPool.empty().add("R", 2).toSnapshot()
      const cost = { generic: 0, C: 1 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(false)
    })

    test("can pay complex multi-color cost", () => {
      const pool = ManaPool.empty()
        .add("W", 1)
        .add("U", 1)
        .add("B", 1)
        .add("R", 1)
        .add("G", 1)
        .add("C", 2)
        .toSnapshot()
      const cost = { generic: 2, W: 1, U: 1, B: 1 }
      expect(ManaPaymentService.canPayCost(pool, cost)).toBe(true)
    })
  })

  describe("payForCost()", () => {
    test("returns same pool for zero cost", () => {
      const pool = ManaPool.empty()
      const cost = { generic: 0 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.equals(pool)).toBe(true)
    })

    test("spends all generic cost from any mana", () => {
      const pool = ManaPool.empty().add("R", 3)
      const cost = { generic: 3 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.isEmpty()).toBe(true)
    })

    test("spends colored mana first", () => {
      const pool = ManaPool.empty().add("U", 2)
      const cost = { generic: 0, U: 2 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.isEmpty()).toBe(true)
    })

    test("spends colored then generic", () => {
      const pool = ManaPool.empty().add("U", 2).add("G", 1).add("C", 1)
      const cost = { generic: 2, U: 2 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.isEmpty()).toBe(true)
    })

    test("prefers colorless for generic cost", () => {
      const pool = ManaPool.empty().add("C", 2).add("G", 1)
      const cost = { generic: 2 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.get("C")).toBe(0)
      expect(result.get("G")).toBe(1)
    })

    test("uses colored mana for generic if colorless insufficient", () => {
      const pool = ManaPool.empty().add("C", 1).add("G", 2)
      const cost = { generic: 3 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.isEmpty()).toBe(true)
    })

    test("uses colored mana in priority order (W, U, B, R, G)", () => {
      const pool = ManaPool.empty()
        .add("W", 1)
        .add("U", 1)
        .add("B", 1)
        .add("R", 1)
        .add("G", 1)
      const cost = { generic: 3 }
      const result = ManaPaymentService.payForCost(pool, cost)
      // Should use W, U, B first (priority order)
      expect(result.get("W")).toBe(0)
      expect(result.get("U")).toBe(0)
      expect(result.get("B")).toBe(0)
      expect(result.get("R")).toBe(1)
      expect(result.get("G")).toBe(1)
    })

    test("spends colorless mana for colorless requirement", () => {
      const pool = ManaPool.empty().add("C", 2).add("R", 1)
      const cost = { generic: 0, C: 1 }
      const result = ManaPaymentService.payForCost(pool, cost)
      expect(result.get("C")).toBe(1)
      expect(result.get("R")).toBe(1)
    })

    test("throws error when insufficient colored mana", () => {
      const pool = ManaPool.empty().add("U", 1)
      const cost = { generic: 0, U: 2 }
      expect(() => ManaPaymentService.payForCost(pool, cost)).toThrow(
        "Insufficient U mana: requested 2, available 1",
      )
    })

    test("throws error when insufficient total mana", () => {
      const pool = ManaPool.empty().add("U", 2).add("G", 1)
      const cost = { generic: 2, U: 2 }
      // After paying U:2, only G:1 remains, need generic:2
      expect(() => ManaPaymentService.payForCost(pool, cost)).toThrow(
        "Insufficient mana to pay generic cost: need 1, available 0",
      )
    })

    test("throws error when insufficient colorless mana", () => {
      const pool = ManaPool.empty().add("R", 2)
      const cost = { generic: 0, C: 1 }
      expect(() => ManaPaymentService.payForCost(pool, cost)).toThrow(
        "Insufficient C mana: requested 1, available 0",
      )
    })

    test("handles complex multi-color payment", () => {
      const pool = ManaPool.empty()
        .add("W", 1)
        .add("U", 1)
        .add("B", 1)
        .add("R", 1)
        .add("G", 1)
        .add("C", 2)
      const cost = { generic: 2, W: 1, U: 1, B: 1 }
      const result = ManaPaymentService.payForCost(pool, cost)
      // Colored: W:1, U:1, B:1 spent
      // Generic: 2 from remaining (prefer C:2, so C is spent)
      // Remaining: R:1, G:1, C:0
      expect(result.get("W")).toBe(0)
      expect(result.get("U")).toBe(0)
      expect(result.get("B")).toBe(0)
      expect(result.get("C")).toBe(0)
      expect(result.total()).toBe(2) // R:1, G:1 left
    })
  })
})
