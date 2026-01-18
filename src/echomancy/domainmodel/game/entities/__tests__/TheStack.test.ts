import { describe, expect, test } from "vitest"
import type { AbilityOnStack, SpellOnStack } from "../../StackTypes"
import { TheStack } from "../TheStack"

// Test fixtures
const createTestSpell = (controllerId: string): SpellOnStack => ({
  kind: "SPELL",
  card: {
    instanceId: `spell-${Math.random()}`,
    definition: {
      id: "test-spell",
      name: "Test Spell",
      types: ["INSTANT"],
    },
    ownerId: controllerId,
  },
  controllerId,
  targets: [],
})

const createTestAbility = (controllerId: string): AbilityOnStack => ({
  kind: "ABILITY",
  sourceId: `source-${Math.random()}`,
  effect: {
    resolve: () => {},
  },
  controllerId,
  targets: [],
})

describe("TheStack Entity", () => {
  describe("empty()", () => {
    test("creates an empty stack", () => {
      const stack = TheStack.empty()

      expect(stack.isEmpty()).toBe(true)
      expect(stack.count()).toBe(0)
      expect(stack.getAll()).toEqual([])
    })
  })

  describe("fromItems()", () => {
    test("creates a stack from an array of items", () => {
      const spell1 = createTestSpell("player-1")
      const spell2 = createTestSpell("player-1")

      const stack = TheStack.fromItems([spell1, spell2])

      expect(stack.count()).toBe(2)
      expect(stack.isEmpty()).toBe(false)
    })

    test("creates a defensive copy of the input array", () => {
      const items = [createTestSpell("player-1")]
      const stack = TheStack.fromItems(items)

      items.push(createTestSpell("player-1"))

      expect(stack.count()).toBe(1)
    })
  })

  describe("push()", () => {
    test("adds an item to the stack", () => {
      const stack = TheStack.empty()
      const spell = createTestSpell("player-1")

      const newStack = stack.push(spell)

      expect(newStack.count()).toBe(1)
      expect(newStack.peek()).toBe(spell)
    })

    test("returns a new instance (immutable pattern)", () => {
      const stack = TheStack.empty()
      const spell = createTestSpell("player-1")

      const newStack = stack.push(spell)

      expect(newStack).not.toBe(stack)
      expect(stack.isEmpty()).toBe(true)
    })

    test("adds items in LIFO order", () => {
      const spell1 = createTestSpell("player-1")
      const spell2 = createTestSpell("player-1")

      const stack = TheStack.empty().push(spell1).push(spell2)

      expect(stack.peek()).toBe(spell2)
    })
  })

  describe("pop()", () => {
    test("removes and returns the top item", () => {
      const spell1 = createTestSpell("player-1")
      const spell2 = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell1, spell2])

      const { item, stack: newStack } = stack.pop()

      expect(item).toBe(spell2)
      expect(newStack.count()).toBe(1)
      expect(newStack.peek()).toBe(spell1)
    })

    test("returns undefined for empty stack", () => {
      const stack = TheStack.empty()

      const { item, stack: newStack } = stack.pop()

      expect(item).toBeUndefined()
      expect(newStack.isEmpty()).toBe(true)
    })

    test("returns a new instance (immutable pattern)", () => {
      const spell = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell])

      const { stack: newStack } = stack.pop()

      expect(newStack).not.toBe(stack)
      expect(stack.count()).toBe(1)
    })
  })

  describe("peek()", () => {
    test("returns the top item without removing it", () => {
      const spell = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell])

      const peeked = stack.peek()

      expect(peeked).toBe(spell)
      expect(stack.count()).toBe(1)
    })

    test("returns undefined for empty stack", () => {
      const stack = TheStack.empty()

      expect(stack.peek()).toBeUndefined()
    })
  })

  describe("getAll()", () => {
    test("returns all items in order (bottom to top)", () => {
      const spell1 = createTestSpell("player-1")
      const spell2 = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell1, spell2])

      const items = stack.getAll()

      expect(items).toEqual([spell1, spell2])
    })

    test("returns a defensive copy", () => {
      const spell = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell])

      const items = stack.getAll()
      items.push(createTestSpell("player-1"))

      expect(stack.count()).toBe(1)
    })
  })

  describe("isEmpty() and hasItems()", () => {
    test("isEmpty returns true for empty stack", () => {
      const stack = TheStack.empty()

      expect(stack.isEmpty()).toBe(true)
      expect(stack.hasItems()).toBe(false)
    })

    test("isEmpty returns false for non-empty stack", () => {
      const stack = TheStack.fromItems([createTestSpell("player-1")])

      expect(stack.isEmpty()).toBe(false)
      expect(stack.hasItems()).toBe(true)
    })
  })

  describe("count()", () => {
    test("returns 0 for empty stack", () => {
      const stack = TheStack.empty()

      expect(stack.count()).toBe(0)
    })

    test("returns correct count for non-empty stack", () => {
      const stack = TheStack.fromItems([
        createTestSpell("player-1"),
        createTestAbility("player-1"),
        createTestSpell("player-2"),
      ])

      expect(stack.count()).toBe(3)
    })
  })

  describe("clear()", () => {
    test("returns an empty stack", () => {
      const stack = TheStack.fromItems([
        createTestSpell("player-1"),
        createTestAbility("player-1"),
      ])

      const cleared = stack.clear()

      expect(cleared.isEmpty()).toBe(true)
      expect(cleared.count()).toBe(0)
    })

    test("original stack is unchanged (immutable)", () => {
      const stack = TheStack.fromItems([createTestSpell("player-1")])

      stack.clear()

      expect(stack.count()).toBe(1)
    })
  })

  describe("items getter (backward compatibility)", () => {
    test("provides access to items array", () => {
      const spell = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell])

      expect(stack.items).toEqual([spell])
    })

    test("allows direct mutation for backward compatibility", () => {
      const spell1 = createTestSpell("player-1")
      const spell2 = createTestSpell("player-1")
      const stack = TheStack.fromItems([spell1])

      stack.items.push(spell2)

      expect(stack.count()).toBe(2)
    })
  })

  describe("mixed spells and abilities", () => {
    test("handles mixed item types correctly", () => {
      const spell = createTestSpell("player-1")
      const ability = createTestAbility("player-2")

      const stack = TheStack.empty().push(spell).push(ability)

      expect(stack.count()).toBe(2)

      const { item: firstPop } = stack.pop()
      expect(firstPop?.kind).toBe("ABILITY")
    })
  })
})
