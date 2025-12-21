# Agent Instructions for Echomancy

## Project Overview

**Echomancy** is a Trading Card Game (TCG) engine built with Next.js 16, React 19, TypeScript, and Tailwind CSS v4.

---

## Critical Rules

### Package Manager
- **USE**: `bun install`, `bun run dev`, `bun test`
- **NEVER**: `npm`, `yarn`, or `pnpm`
- **NEVER**: commit `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`

### Before Writing Code
1. Read `src/echomancy/domainmodel/game/Game.ts` - this is the core
2. Read `src/echomancy/domainmodel/game/__tests__/helpers.ts` before writing tests
3. Check existing tests for patterns to follow

### Before Committing
1. Run `bun test` - all tests must pass
2. Run `bun run lint` - no linting errors
3. Run `bun run format` - code must be formatted

---

## Game Engine Rules

### ALWAYS: Use `enterBattlefield()` for Permanents
All permanents entering the battlefield MUST go through `game.enterBattlefield()`. This ensures ETB triggers fire and creature state initializes correctly.

```typescript
// CORRECT
game.enterBattlefield(permanent, controllerId)

// WRONG - will break ETB triggers
playerState.battlefield.cards.push(permanent)
```

### ALWAYS: Use Test Helpers
Never manually set up game state in tests. Use helpers from `helpers.ts`:

- `createStartedGame()` - creates a 2-player game
- `addCreatureToBattlefield()` - adds creature properly
- `advanceToStep()` - moves game to specific phase
- `resolveStack()` - resolves spells on stack

### ALWAYS: Resolve the Stack
After casting a spell, remember to resolve it before asserting results:

```typescript
game.apply({ type: "CAST_SPELL", ... })
resolveStack(game, opponentId, playerId)  // Don't forget this!
// Now assert on the resolved state
```

### ALWAYS: Use `game.apply()` for Mutations
All game state changes go through actions. Never mutate state directly.

---

## Code Style

### TypeScript
- Use `type` over `interface`
- Use `@/*` path alias for imports from `src/`
- Strict mode is enabled - no `any` types

### React
- Server Components by default
- `"use client"` only when needed
- React Compiler is enabled - no manual memoization needed

### Patterns
- Use `ts-pattern` for exhaustive action matching
- Follow existing Effect implementations when adding new effects
- Errors are domain-specific classes in `GameErrors.ts`

---

## When Unsure

### Ask Before
- Adding new action types to `Game.apply()`
- Modifying the Step/phase system
- Changing how the stack works
- Adding new card types

### Reference First
- For new Effects: look at `src/echomancy/domainmodel/effects/impl/`
- For new tests: look at existing test files for patterns
- For card behavior: check if similar card types exist

---

## Commands Reference

```bash
bun run dev      # Start dev server
bun run build    # Production build
bun test         # Run tests
bun run lint     # Check linting
bun run format   # Format code
```
