# Testing Guide

This guide covers testing patterns, helpers, and best practices for the Echomancy game engine.

## Running Tests

```bash
# Run all tests
bun test

# Run tests matching a pattern
bun test <name>

# Run specific test file
bun test Game.triggers

# Watch mode
bun test --watch
```

## Test Helpers

Test helpers are located in `src/echomancy/domainmodel/game/__tests__/helpers.ts`. Always use these instead of manual setup.

### Game Setup

#### createStartedGame()

Creates a 2-player game in the UNTAP step.

```typescript
import { createStartedGame } from "./helpers"

const { game, player1, player2, dummyLandInstanceId } = createStartedGame()
```

Returns:
- `game`: Game instance
- `player1`, `player2`: Player instances
- `dummyLandInstanceId`: ID of a dummy land in player1's hand

#### createGameInMainPhase()

Creates a game advanced to FIRST_MAIN phase.

```typescript
import { createGameInMainPhase } from "./helpers"

const { game, player1, player2 } = createGameInMainPhase()
// game.currentStep === Step.FIRST_MAIN
```

### Step Navigation

#### advanceToStep()

Advances the game to a specific step.

```typescript
import { advanceToStep } from "./helpers"
import { Step } from "../Steps"

advanceToStep(game, Step.DECLARE_ATTACKERS)
```

### Card Creation

#### createTestCreature()

Creates a basic creature card.

```typescript
import { createTestCreature } from "./helpers"

const creature = createTestCreature(playerId)
// Or with specific ID
const creature = createTestCreature(playerId, "my-creature-id")
```

#### createTestSpell()

Creates a basic instant spell.

```typescript
import { createTestSpell } from "./helpers"

const spell = createTestSpell(playerId)
const spell = createTestSpell(playerId, "spell-1")
```

#### createCreatureWithETBTrigger()

Creates a creature with a custom ETB callback.

```typescript
import { createCreatureWithETBTrigger } from "./helpers"

let etbFired = false
const creature = createCreatureWithETBTrigger(
  "creature-id",
  playerId,
  () => { etbFired = true }
)

addCreatureToBattlefield(game, playerId, creature)
expect(etbFired).toBe(true)
```

### Themed Card Helpers

For trigger system testing:

```typescript
import {
  createElvishVisionary,    // ETB: draw a card
  createLlanowarElves,      // Vanilla elf (mana ability TODO)
  createElvishWarrior,      // Vanilla elf
  createConditionalElf,     // ETB if you control another elf
  createElfWithAttackTrigger // Attack trigger: draw a card
} from "./helpers"
```

### Zone Manipulation

#### addCreatureToBattlefield()

**IMPORTANT**: Uses `game.enterBattlefield()` internally to ensure ETB triggers fire.

```typescript
import { addCreatureToBattlefield } from "./helpers"

const creature = createTestCreature(playerId)
addCreatureToBattlefield(game, playerId, creature)
// ETB triggers will fire!
```

#### addSpellToHand()

```typescript
import { addSpellToHand } from "./helpers"

const spell = createTestSpell(playerId)
addSpellToHand(game, playerId, spell)
```

#### addCreatureToHand()

```typescript
import { addCreatureToHand } from "./helpers"

const creature = createTestCreature(playerId)
addCreatureToHand(game, playerId, creature)
```

### Combat Setup

#### setupCreatureInCombat()

Creates a creature and advances to DECLARE_ATTACKERS.

```typescript
import { setupCreatureInCombat } from "./helpers"

const creature = setupCreatureInCombat(game, playerId)
// game.currentStep === Step.DECLARE_ATTACKERS
// creature is on battlefield, ready to attack
```

#### setupMultipleCreatures()

Creates multiple creatures on the battlefield.

```typescript
import { setupMultipleCreatures } from "./helpers"

const creatures = setupMultipleCreatures(game, playerId, 3)
// creatures.length === 3
```

### Stack Resolution

#### resolveStack()

Makes both players pass priority to resolve the top stack item.

```typescript
import { resolveStack } from "./helpers"

game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
resolveStack(game, opponentId, playerId)
expect(game.getStack()).toHaveLength(0)
```

#### assertSpellAt() / assertAbilityAt()

Type-safe stack inspection.

```typescript
import { assertSpellAt, assertAbilityAt } from "./helpers"

const spell = assertSpellAt(game.getStack(), 0)
// spell is typed as SpellOnStack

const ability = assertAbilityAt(game.getStack(), 0)
// ability is typed as AbilityOnStack
```

### Extra Phases

#### scheduleExtraCombatPhase()

```typescript
import { scheduleExtraCombatPhase } from "./helpers"

scheduleExtraCombatPhase(game)
// Extra combat steps will occur after current sequence
```

## Testing Patterns

### Basic Test Structure

```typescript
import { describe, it, expect } from "vitest"
import { createStartedGame, advanceToStep } from "./helpers"
import { Step } from "../Steps"

describe("Feature", () => {
  it("should do something", () => {
    // Arrange
    const { game, player1 } = createStartedGame()
    advanceToStep(game, Step.FIRST_MAIN)

    // Act
    game.apply({ type: "SOME_ACTION", playerId: player1.id })

    // Assert
    expect(game.someState).toBe(expectedValue)
  })
})
```

### Testing Triggers

```typescript
it("should fire ETB trigger", () => {
  const { game, player1 } = createGameInMainPhase()

  let triggerFired = false
  const creature = createCreatureWithETBTrigger(
    "creature-1",
    player1.id,
    () => { triggerFired = true }
  )

  addCreatureToBattlefield(game, player1.id, creature)

  expect(triggerFired).toBe(true)
})
```

### Testing Stack Resolution

```typescript
it("should resolve spell from stack", () => {
  const { game, player1, player2 } = createGameInMainPhase()

  const spell = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spell)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: []
  })

  expect(game.getStack()).toHaveLength(1)

  resolveStack(game, player2.id, player1.id)

  expect(game.getStack()).toHaveLength(0)
})
```

### Testing Errors

```typescript
import { LandLimitExceededError } from "../GameErrors"

it("should throw when playing second land", () => {
  const { game, player1, dummyLandInstanceId } = createGameInMainPhase()

  // Play first land
  game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: dummyLandInstanceId })

  // Create second land
  const secondLand = createLand(player1.id, "land-2")
  addCardToHand(game, player1.id, secondLand)

  // Should throw
  expect(() => {
    game.apply({ type: "PLAY_LAND", playerId: player1.id, cardId: "land-2" })
  }).toThrow(LandLimitExceededError)
})
```

## Red Flags

Avoid these patterns in tests:

| Red Flag | Use Instead |
|----------|-------------|
| `new Game(...)` | `createStartedGame()` |
| `battlefield.cards.push(...)` | `addCreatureToBattlefield()` |
| Manual step loop | `advanceToStep()` |
| Missing `resolveStack()` | Always resolve before asserting effects |
| Asserting before resolution | Resolve stack first |

## Test Organization

Tests are organized by feature:

```
game/__tests__/
├── helpers.ts                              # All test helpers
├── Game.test.ts                            # Core game mechanics
├── Game.triggers.test.ts                   # Trigger system
├── Game.activatedAbilities.test.ts         # Activated abilities
├── Game.priorityAndStackResolution.test.ts # Stack and priority
├── Game.enterBattlefield.test.ts           # ETB mechanics
├── Game.declareAttacker.test.ts            # Combat attacks
├── Game.castSpell.test.ts                  # Spell casting
├── Game.playLand.test.ts                   # Land playing
└── ...
```

## Debugging Tips

### Check Stack State

```typescript
console.log("Stack:", game.getStack().map(item => ({
  kind: item.kind,
  name: item.kind === "SPELL" ? item.card.definition.name : "ability"
})))
```

### Check Player State

```typescript
const state = game.getPlayerState(playerId)
console.log("Hand:", state.hand.cards.map(c => c.definition.name))
console.log("Battlefield:", state.battlefield.cards.map(c => c.definition.name))
```

### Trace Step Changes

```typescript
console.log("Current step:", game.currentStep)
console.log("Current player:", game.currentPlayerId)
```

## Before Committing

Always run the full test suite:

```bash
bun test && bun run lint && bun run format
```

All three must pass before committing.
