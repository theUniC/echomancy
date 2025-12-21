# Agent Instructions for Echomancy

**Echomancy** is a Trading Card Game (TCG) engine built with Next.js 16, React 19, TypeScript, and Tailwind CSS v4.

---

## Task Workflow

When starting any task, follow this order:

1. **Understand** - Read relevant source files before writing code
2. **Check patterns** - Look at existing similar code/tests
3. **Implement** - Write the code
4. **Test** - Run `bun test`
5. **Lint** - Run `bun run lint && bun run format`
6. **Commit** - Only if steps 4-5 pass

---

## P0: Never Violate

These rules exist to prevent silent bugs that pass tests but break game logic.

### Use `enterBattlefield()` for all permanents

```typescript
// CORRECT
game.enterBattlefield(permanent, controllerId)

// WRONG - ETB triggers won't fire, creature state won't initialize
playerState.battlefield.cards.push(permanent)
```

> **Why**: Direct array push bypasses the ETB system. Tests may pass but the game will behave incorrectly in production.

### Use `game.apply()` for all state mutations

```typescript
// CORRECT
game.apply({ type: "PLAY_LAND", playerId, cardId })

// WRONG - bypasses validation, breaks game rules
playerState.hand.cards.splice(index, 1)
```

> **Why**: `apply()` validates game rules (timing, permissions, costs). Direct mutation creates illegal game states.

### Use Bun, never npm/yarn/pnpm

```bash
# CORRECT
bun install
bun test
bun run dev

# WRONG - creates conflicting lockfiles
npm install
```

> **Why**: Mixed package managers cause dependency conflicts and CI failures.

---

## P1: Strong Preferences

Violate only with explicit justification.

### Use test helpers, not manual setup

```typescript
// CORRECT
const { game, player1 } = createStartedGame()
const creature = createTestCreature(player1.id)
addCreatureToBattlefield(game, player1.id, creature)

// AVOID - error-prone, verbose, may miss setup steps
const game = new Game(...)
const creature = { instanceId: "c1", ... }
playerState.battlefield.cards.push(creature)
```

> **Why**: Helpers ensure consistent setup and use `enterBattlefield()` internally.

### Resolve the stack before asserting

```typescript
// CORRECT
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
resolveStack(game, opponentId, playerId)
expect(game.getStack()).toHaveLength(0)

// WRONG - asserting on unresolved state
game.apply({ type: "CAST_SPELL", ... })
expect(somethingAboutTheResolvedEffect) // Will fail!
```

### Read before writing

Before implementing anything in the game engine:
1. Read `src/echomancy/domainmodel/game/Game.ts`
2. Read `src/echomancy/domainmodel/game/__tests__/helpers.ts`
3. Check existing tests for the pattern you need

### Run checks before committing

```bash
bun test && bun run lint && bun run format
```

All three must pass. No exceptions.

---

## P2: Style Guidelines

Follow for consistency. Can be adjusted if codebase evolves.

### TypeScript
- Use `type` over `interface`
- Use `@/*` path alias for imports from `src/`
- No `any` types (strict mode is enabled)

### React
- Server Components by default
- `"use client"` only when state/effects/browser APIs needed
- No manual `useMemo`/`useCallback` (React Compiler handles it)

### Game Engine
- Use `ts-pattern` with `.exhaustive()` for action matching
- New Effects go in `src/echomancy/domainmodel/effects/impl/`
- Domain errors extend classes in `GameErrors.ts`

---

## Red Flags

If you see yourself doing any of these, stop and reconsider:

| Red Flag | What to do instead |
|----------|---------------------|
| `battlefield.cards.push(...)` | Use `enterBattlefield()` |
| `hand.cards.splice(...)` | Use `game.apply()` with appropriate action |
| `new Game(...)` in tests | Use `createStartedGame()` |
| `npm install` or `yarn add` | Use `bun install` or `bun add` |
| Using `any` type | Find or create proper type |
| Skipping `resolveStack()` | Always resolve before asserting effects |
| Committing without `bun test` | Run tests first |

---

## Before Modifying Core Systems

These areas require extra caution. Read extensively before changing:

| System | Key files to read first |
|--------|------------------------|
| Actions/Commands | `Game.ts` (apply method, action types) |
| Turn structure | `Steps.ts`, `StepMachine.ts` |
| Stack resolution | `Game.ts` (resolveTopOfStack, passPriority) |
| New card types | `CardDefinition.ts`, existing tests |
| New effects | `Effect.ts`, `effects/impl/*` |

If the change feels risky, write tests first to lock current behavior.

---

## Commands

```bash
bun run dev      # Dev server at localhost:3000
bun run build    # Production build
bun test         # Run all tests
bun test <name>  # Run tests matching name
bun run lint     # Check code style
bun run format   # Auto-format code
```

---

## File Locations

| What | Where |
|------|-------|
| Game engine core | `src/echomancy/domainmodel/game/` |
| Card/Effect types | `src/echomancy/domainmodel/cards/`, `effects/` |
| Test helpers | `src/echomancy/domainmodel/game/__tests__/helpers.ts` |
| All tests | `src/echomancy/domainmodel/game/__tests__/` |
