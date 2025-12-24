import { expect, test } from "vitest"
import {
  addCreatureToBattlefield,
  createStartedGame,
  createTestCreature,
} from "./helpers"

/**
 * Tests for Power/Toughness + Counters MVP
 *
 * This test suite validates the creature stats system as defined in the
 * Power/Toughness + Counters MVP Contract.
 *
 * Scope:
 * - Base power/toughness initialization
 * - Counter addition and removal
 * - Current power/toughness calculation
 *
 * Explicitly excluded from MVP:
 * - Damage tracking
 * - Static abilities
 * - Continuous effects
 * - Layer system
 * - Temporary modifiers
 */

// ============================================================================
// Rule 1 — Base Power and Toughness Initialization
// ============================================================================

test("creature initializes with correct base power and toughness", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 3, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getBasePower(creature.instanceId)).toBe(3)
  expect(game.getBaseToughness(creature.instanceId)).toBe(2)
})

test("creature with undefined power/toughness defaults to 0/1", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getBasePower(creature.instanceId)).toBe(0)
  expect(game.getBaseToughness(creature.instanceId)).toBe(1)
})

test("creature with explicit 0/0 stats initializes correctly", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 0, 0)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getBasePower(creature.instanceId)).toBe(0)
  expect(game.getBaseToughness(creature.instanceId)).toBe(0)
})

test("large creature stats are handled correctly", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 20, 20)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getBasePower(creature.instanceId)).toBe(20)
  expect(game.getBaseToughness(creature.instanceId)).toBe(20)
})

// ============================================================================
// Rule 2 — Counter Initialization
// ============================================================================

test("creature starts with zero +1/+1 counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

// ============================================================================
// Rule 3 — Adding Counters
// ============================================================================

test("adding +1/+1 counters increases counter count", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(3)
})

test("adding counters multiple times accumulates correctly", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(5)
})

test("adding zero or negative counters throws error", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(() => {
    game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 0)
  }).toThrow("Amount must be positive")

  expect(() => {
    game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", -1)
  }).toThrow("Amount must be positive")
})

// ============================================================================
// Rule 4 — Removing Counters
// ============================================================================

test("removing counters decreases counter count", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 5)
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(3)
})

test("removing all counters results in zero counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

test("removing more counters than exist clamps to zero", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 5)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

test("removing counters from creature with zero counters is safe", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

test("removing zero or negative counters throws error", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(() => {
    game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 0)
  }).toThrow("Amount must be positive")

  expect(() => {
    game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", -1)
  }).toThrow("Amount must be positive")
})

// ============================================================================
// Rule 5 — Current Power Calculation
// ============================================================================

test("current power equals base power when no counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 3, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getCurrentPower(creature.instanceId)).toBe(3)
})

test("current power includes +1/+1 counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCurrentPower(creature.instanceId)).toBe(5) // 2 + 3
})

test("current power calculation with zero base power", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 0, 1)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 4)

  expect(game.getCurrentPower(creature.instanceId)).toBe(4) // 0 + 4
})

test("current power updates when counters are added or removed", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  // Add counters
  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)
  expect(game.getCurrentPower(creature.instanceId)).toBe(5)

  // Add more counters
  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  expect(game.getCurrentPower(creature.instanceId)).toBe(7)

  // Remove some counters
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 4)
  expect(game.getCurrentPower(creature.instanceId)).toBe(3)
})

// ============================================================================
// Rule 6 — Current Toughness Calculation
// ============================================================================

test("current toughness equals base toughness when no counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 3, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  expect(game.getCurrentToughness(creature.instanceId)).toBe(2)
})

test("current toughness includes +1/+1 counters", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCurrentToughness(creature.instanceId)).toBe(5) // 2 + 3
})

test("current toughness calculation with zero base toughness", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 1, 0)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 4)

  expect(game.getCurrentToughness(creature.instanceId)).toBe(4) // 0 + 4
})

test("current toughness updates when counters are added or removed", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  // Add counters
  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)
  expect(game.getCurrentToughness(creature.instanceId)).toBe(5)

  // Add more counters
  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  expect(game.getCurrentToughness(creature.instanceId)).toBe(7)

  // Remove some counters
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 4)
  expect(game.getCurrentToughness(creature.instanceId)).toBe(3)
})

// ============================================================================
// Rule 7 — Power and Toughness are Independent
// ============================================================================

test("+1/+1 counters affect both power and toughness equally", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 3)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 4)

  expect(game.getCurrentPower(creature.instanceId)).toBe(6) // 2 + 4
  expect(game.getCurrentToughness(creature.instanceId)).toBe(7) // 3 + 4
})

test("base power and toughness remain unchanged after counter manipulation", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 3, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 5)
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)

  // Base stats should not change
  expect(game.getBasePower(creature.instanceId)).toBe(3)
  expect(game.getBaseToughness(creature.instanceId)).toBe(2)
})

// ============================================================================
// Rule 8 — Multiple Creatures are Independent
// ============================================================================

test("counters on one creature do not affect another", () => {
  const { game, player1 } = createStartedGame()
  const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
  const creature2 = createTestCreature(player1.id, "creature-2", 3, 3)

  addCreatureToBattlefield(game, player1.id, creature1)
  addCreatureToBattlefield(game, player1.id, creature2)

  game.addCounters(creature1.instanceId, "PLUS_ONE_PLUS_ONE", 4)

  expect(game.getCurrentPower(creature1.instanceId)).toBe(6)
  expect(game.getCurrentPower(creature2.instanceId)).toBe(3)
  expect(game.getCounters(creature2.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

test("multiple creatures can have different counter counts", () => {
  const { game, player1 } = createStartedGame()
  const creature1 = createTestCreature(player1.id, "creature-1", 1, 1)
  const creature2 = createTestCreature(player1.id, "creature-2", 2, 2)
  const creature3 = createTestCreature(player1.id, "creature-3", 3, 3)

  addCreatureToBattlefield(game, player1.id, creature1)
  addCreatureToBattlefield(game, player1.id, creature2)
  addCreatureToBattlefield(game, player1.id, creature3)

  game.addCounters(creature1.instanceId, "PLUS_ONE_PLUS_ONE", 1)
  game.addCounters(creature2.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  game.addCounters(creature3.instanceId, "PLUS_ONE_PLUS_ONE", 3)

  expect(game.getCurrentPower(creature1.instanceId)).toBe(2)
  expect(game.getCurrentPower(creature2.instanceId)).toBe(4)
  expect(game.getCurrentPower(creature3.instanceId)).toBe(6)
})

// ============================================================================
// Rule 9 — Invariants
// ============================================================================

test("counter count never goes negative", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id, "creature-1", 2, 2)
  addCreatureToBattlefield(game, player1.id, creature)

  // Remove more counters than exist
  game.removeCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 100)

  expect(game.getCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE")).toBe(0)
})

test("identical creatures produce identical results", () => {
  const { game, player1 } = createStartedGame()
  const creature1 = createTestCreature(player1.id, "creature-1", 3, 4)
  const creature2 = createTestCreature(player1.id, "creature-2", 3, 4)

  addCreatureToBattlefield(game, player1.id, creature1)
  addCreatureToBattlefield(game, player1.id, creature2)

  // Apply same counter operations
  game.addCounters(creature1.instanceId, "PLUS_ONE_PLUS_ONE", 2)
  game.addCounters(creature2.instanceId, "PLUS_ONE_PLUS_ONE", 2)

  expect(game.getCurrentPower(creature1.instanceId)).toBe(
    game.getCurrentPower(creature2.instanceId),
  )
  expect(game.getCurrentToughness(creature1.instanceId)).toBe(
    game.getCurrentToughness(creature2.instanceId),
  )
})

// ============================================================================
// Rule 10 — Error Handling
// ============================================================================

test("accessing stats of non-existent creature throws error", () => {
  const { game } = createStartedGame()

  expect(() => {
    game.getBasePower("non-existent-creature")
  }).toThrow()

  expect(() => {
    game.getCurrentPower("non-existent-creature")
  }).toThrow()

  expect(() => {
    game.addCounters("non-existent-creature", "PLUS_ONE_PLUS_ONE", 1)
  }).toThrow()
})
