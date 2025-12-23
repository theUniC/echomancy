# Testing Guide

This guide covers testing patterns, helpers, and best practices for the Echomancy game engine.

## Running Tests

Use Bun for all test operations:
- `bun test` runs all tests
- `bun test <pattern>` runs tests matching the pattern
- `bun test --watch` runs in watch mode

Always run `bun test && bun run lint && bun run format` before committing.

## Test Helpers

Test helpers are located in the `__tests__/helpers.ts` file. Always use these instead of manual setup to ensure consistent game state initialization.

### Game Setup Helpers

**createStartedGame()** creates a 2-player game in the UNTAP step. Returns the game, both players, and a dummy land ID for testing land plays.

**createGameInMainPhase()** creates a game and advances it to FIRST_MAIN, where most actions are legal.

**advanceToStep()** advances the game to any specified step, handling all intermediate step transitions.

### Card Creation Helpers

**createTestCreature()** creates a basic creature card with an owner.

**createTestSpell()** creates a basic instant spell.

**createCreatureWithETBTrigger()** creates a creature with a callback that fires on enter-the-battlefield. Useful for verifying trigger execution.

**Themed card helpers** (createElvishVisionary, createLlanowarElves, etc.) create specific card configurations for testing trigger scenarios.

### Zone Manipulation Helpers

**addCreatureToBattlefield()** puts a creature on the battlefield using the proper `game.enterBattlefield()` method. This ensures ETB triggers fire correctly.

**addSpellToHand()** adds a spell to a player's hand for casting.

**addCreatureToHand()** adds a creature to a player's hand.

### Combat Helpers

**setupCreatureInCombat()** creates a creature, adds it to the battlefield, and advances to DECLARE_ATTACKERS step.

**setupMultipleCreatures()** creates and adds multiple creatures to the battlefield.

### Stack Helpers

**resolveStack()** makes both players pass priority, causing the top stack item to resolve. Essential for testing spell and ability resolution.

**assertSpellAt()** and **assertAbilityAt()** provide type-safe inspection of stack items.

### Extra Phase Helpers

**scheduleExtraCombatPhase()** schedules an additional combat phase, useful for testing effects that grant extra combats.

## Testing Patterns

### Basic Test Structure

Follow Arrange-Act-Assert:
1. Set up the game state using helpers
2. Perform the action being tested
3. Assert the expected outcome

### Testing Triggers

Create a creature with a trigger callback that sets a flag or captures data. Add the creature to the battlefield. Verify the callback was invoked.

### Testing Stack Resolution

Cast a spell or activate an ability. Verify it's on the stack. Call resolveStack(). Verify the stack is empty and the effect occurred.

### Testing Errors

Use expect().toThrow() to verify that invalid actions throw the appropriate error type.

## Common Mistakes to Avoid

**Don't use `new Game()` directly.** Use the helper functions which set up proper initial state.

**Don't push directly to battlefield arrays.** Use `addCreatureToBattlefield()` which calls `enterBattlefield()` properly.

**Don't forget to resolve the stack.** If you're testing what happens when a spell resolves, you must call resolveStack() before asserting.

**Don't assert on unresolved state.** The spell's effect hasn't happened until the stack resolves.

## Test Organization

Tests are organized by feature:
- Game.test.ts - Core game mechanics
- Game.triggers.test.ts - Trigger system
- Game.activatedAbilities.test.ts - Activated abilities
- Game.priorityAndStackResolution.test.ts - Stack and priority
- Game.enterBattlefield.test.ts - ETB mechanics
- Game.declareAttacker.test.ts - Combat attacks
- Game.castSpell.test.ts - Spell casting
- Game.playLand.test.ts - Land playing

## Debugging Tips

If a test fails unexpectedly:
1. Check if the stack was resolved before asserting
2. Verify the game is in the correct step for the action
3. Confirm creatures were added via enterBattlefield, not direct array push
4. Check that the correct player is taking the action

## Before Committing

Run the full validation suite:
- `bun test` - All tests must pass
- `bun run lint` - No linting errors
- `bun run format` - Code properly formatted

All three must pass before any commit.
