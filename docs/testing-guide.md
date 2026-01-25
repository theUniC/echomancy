# Testing Guide

Testing patterns and helpers for the Echomancy game engine.

## Key Concepts

- **Test Helpers** - Reusable functions in `__tests__/helpers.ts` for consistent setup
- **Arrange-Act-Assert** - Standard test structure (setup, action, verify)
- **Stack Resolution** - Must call `resolveStack()` before asserting on spell effects
- **No Mocks** - Use actual Game class, not mocks

## How It Works

**Running Tests**:
- `bun test` - Run all tests
- `bun test <pattern>` - Run tests matching pattern
- `bun test --watch` - Watch mode

**Before Committing**: Always run `bun test && bun run lint && bun run format`

**Test Helpers** (see `__tests__/helpers.ts` for implementations):

**Game Setup**:
- `createStartedGame()` - 2-player game in UNTAP step
- `createGameInMainPhase()` - Game advanced to FIRST_MAIN
- `advanceToStep()` - Advance to any step

**Card Creation**:
- `createTestCreature()` - Basic creature
- `createTestSpell()` - Basic instant
- `createCreatureWithETBTrigger()` - Creature with ETB callback
- Themed helpers: `createElvishVisionary()`, `createLlanowarElves()`, etc.

**Zone Manipulation**:
- `addCreatureToBattlefield()` - Properly adds creature (fires ETB triggers)
- `addSpellToHand()` - Add spell to hand
- `addCreatureToHand()` - Add creature to hand

**Combat**:
- `setupCreatureInCombat()` - Create creature and advance to DECLARE_ATTACKERS
- `setupMultipleCreatures()` - Add multiple creatures to battlefield

**Stack**:
- `resolveStack()` - Both players pass, top item resolves
- `assertSpellAt()` - Type-safe stack inspection for spells
- `assertAbilityAt()` - Type-safe stack inspection for abilities

**Extra Phases**:
- `scheduleExtraCombatPhase()` - Schedule additional combat phase

## Rules

- Always use helpers instead of manual setup
- Never push directly to battlefield arrays (use `addCreatureToBattlefield()`)
- Always resolve stack before asserting on spell effects
- Use `expect().toThrow()` for error validation
- Follow Arrange-Act-Assert pattern
- One behavior per test

## Common Mistakes

- Using `new Game()` directly (use helpers)
- Pushing to battlefield without calling `enterBattlefield()`
- Asserting before calling `resolveStack()`
- Testing multiple unrelated behaviors in one test

**Test Organization**: Tests are in `__tests__/` organized by feature (Game.test.ts, Game.triggers.test.ts, etc.)
