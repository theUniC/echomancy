import { describe, expect, test } from "vitest"
import { CardCatalog } from "../CardCatalog"

describe("CardCatalog", () => {
  describe("Basic Lands", () => {
    test("Forest is a land with no mana cost", () => {
      const forest = CardCatalog.Forest

      expect(forest.id).toBe("forest")
      expect(forest.name).toBe("Forest")
      expect(forest.types).toEqual(["LAND"])
      expect(forest.manaCost).toBeUndefined()
    })

    test("Mountain is a land with no mana cost", () => {
      const mountain = CardCatalog.Mountain

      expect(mountain.id).toBe("mountain")
      expect(mountain.name).toBe("Mountain")
      expect(mountain.types).toEqual(["LAND"])
      expect(mountain.manaCost).toBeUndefined()
    })

    test("Plains is a land with no mana cost", () => {
      const plains = CardCatalog.Plains

      expect(plains.id).toBe("plains")
      expect(plains.name).toBe("Plains")
      expect(plains.types).toEqual(["LAND"])
      expect(plains.manaCost).toBeUndefined()
    })

    test("Island is a land with no mana cost", () => {
      const island = CardCatalog.Island

      expect(island.id).toBe("island")
      expect(island.name).toBe("Island")
      expect(island.types).toEqual(["LAND"])
      expect(island.manaCost).toBeUndefined()
    })

    test("Swamp is a land with no mana cost", () => {
      const swamp = CardCatalog.Swamp

      expect(swamp.id).toBe("swamp")
      expect(swamp.name).toBe("Swamp")
      expect(swamp.types).toEqual(["LAND"])
      expect(swamp.manaCost).toBeUndefined()
    })
  })

  describe("Creatures", () => {
    test("Bear is a 2/2 creature", () => {
      const bear = CardCatalog.Bear

      expect(bear.id).toBe("bear")
      expect(bear.name).toBe("Bear")
      expect(bear.types).toEqual(["CREATURE"])
      expect(bear.power).toBe(2)
      expect(bear.toughness).toBe(2)
    })

    test("Elite Vanguard is a 2/1 creature", () => {
      const vanguard = CardCatalog.EliteVanguard

      expect(vanguard.id).toBe("elite-vanguard")
      expect(vanguard.name).toBe("Elite Vanguard")
      expect(vanguard.types).toEqual(["CREATURE"])
      expect(vanguard.power).toBe(2)
      expect(vanguard.toughness).toBe(1)
    })
  })

  describe("Spells", () => {
    test("Giant Growth is an instant", () => {
      const growth = CardCatalog.GiantGrowth

      expect(growth.id).toBe("giant-growth")
      expect(growth.name).toBe("Giant Growth")
      expect(growth.types).toEqual(["INSTANT"])
    })

    test("Lightning Strike is an instant", () => {
      const strike = CardCatalog.LightningStrike

      expect(strike.id).toBe("lightning-strike")
      expect(strike.name).toBe("Lightning Strike")
      expect(strike.types).toEqual(["INSTANT"])
    })

    test("Divination is a sorcery", () => {
      const divination = CardCatalog.Divination

      expect(divination.id).toBe("divination")
      expect(divination.name).toBe("Divination")
      expect(divination.types).toEqual(["SORCERY"])
    })
  })
})
