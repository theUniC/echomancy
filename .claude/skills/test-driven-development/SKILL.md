---
name: test-driven-development
description: Use when implementing any feature or bugfix, before writing implementation code (project)
---

# Test-Driven Development (TDD)

Write the test first. Watch it fail. Write minimal code to pass.

## Project Context

**Test helpers** (use these, don't reinvent):
```typescript
import { createStartedGame, createTestCreature, addCreatureToBattlefield, resolveStack } from './helpers'

const { game, player1, player2 } = createStartedGame()
const creature = createTestCreature(player1.id)
addCreatureToBattlefield(game, player1.id, creature)
```

**Always resolve stack before asserting:**
```typescript
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
resolveStack(game, player2.id, player1.id)
expect(game.getStack()).toHaveLength(0)
```

**Run tests:**
```bash
bun test path/to/test.test.ts
```

## The Cycle: Red → Green → Refactor

### 1. RED: Write failing test

```typescript
test('creature with flying can only be blocked by creatures with flying or reach', () => {
  const { game, player1, player2 } = createStartedGame()
  const flyer = createTestCreature(player1.id, { keywords: ['flying'] })
  const groundBlocker = createTestCreature(player2.id)

  addCreatureToBattlefield(game, player1.id, flyer)
  addCreatureToBattlefield(game, player2.id, groundBlocker)

  // Declare flyer as attacker
  game.apply({ type: "DECLARE_ATTACKERS", playerId: player1.id, attackers: [flyer.id] })

  // Try to block with ground creature - should fail
  expect(() => {
    game.apply({ type: "DECLARE_BLOCKERS", playerId: player2.id, blockers: { [flyer.id]: groundBlocker.id } })
  }).toThrow()
})
```

**Run it. Confirm it fails for the right reason.**

### 2. GREEN: Minimal code to pass

Write the simplest code that makes the test pass. No extras.

### 3. REFACTOR: Clean up

Only after green. Keep tests passing.

### 4. Repeat

Next test for next behavior.

## Good Tests

- **One behavior per test** - If name has "and", split it
- **Clear name** - Describes what should happen
- **Real code** - Use actual `Game`, not mocks
- **Use helpers** - `createStartedGame()`, `createTestCreature()`, etc.

## Red Flags

Stop and fix if:
- Test passes immediately (you're testing existing behavior)
- Test errors instead of fails (fix the error first)
- Writing code before test exists

## Checklist

Before done:
- [ ] Test existed and failed before implementation
- [ ] Test failed for the right reason
- [ ] Wrote minimal code to pass
- [ ] All tests pass: `bun test`
