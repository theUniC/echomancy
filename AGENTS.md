# Agent Instructions for Echomancy

## Project Overview

**Echomancy** is a Next.js 16 application using the App Router with React 19, TypeScript, and Tailwind CSS v4. This is currently a fresh project bootstrapped with `create-next-app`.

## Tech Stack & Key Dependencies

- **Next.js 16.0.7** with App Router (`src/app/` directory structure)
- **React 19.2.0** with **React Compiler** enabled (`reactCompiler: true` in `next.config.ts`)
- **TypeScript 5** with strict mode enabled
- **Tailwind CSS v4** (new version with PostCSS plugin `@tailwindcss/postcss`)
- **Biome** for linting and formatting (replaces ESLint + Prettier)
- **Bun** as the package manager and runtime

## Package Manager

**IMPORTANT: This project uses Bun exclusively.**

- ✅ **USE**: `bun install`, `bun run dev`, `bun test`, etc.
- ❌ **DO NOT USE**: `npm`, `yarn`, or `pnpm` commands
- ❌ **NEVER** commit `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`
- ✅ **ONLY** commit `bun.lockb` for dependency locking

If you accidentally run npm commands, delete any generated `package-lock.json` files immediately.

## Development Workflow

### Commands
```bash
bun run dev      # Start development server (localhost:3000)
bun run build    # Production build
bun run start    # Run production build
bun run lint     # Run Biome linter checks
bun run format   # Format code with Biome
bun test         # Run tests with Vitest
```

### Code Quality
- Use **Biome** for all linting and formatting tasks
- Run `bun run format` before committing to ensure consistent code style
- Biome is configured in `biome.json` with Next.js and React recommended rules

## Project Structure

```
src/
  app/
    layout.tsx       # Root layout with Geist fonts
    page.tsx         # Home page
    globals.css      # Global styles with Tailwind v4 @theme
public/             # Static assets
```

## Code Conventions

### TypeScript
- **Strict mode enabled** - all type safety features are on
- Use `@/*` path alias for imports from `src/` (e.g., `import { Foo } from "@/components/foo"`)
- Prefer explicit types for component props and function parameters
- Use `type` over `interface` for consistency with existing code

### Styling with Tailwind CSS v4
- **New Tailwind v4 syntax**: Use `@import "tailwindcss"` in CSS (see `globals.css`)
- **Inline themes**: Use `@theme inline { }` blocks for custom CSS variables
- Global design tokens defined in `:root` with CSS variables (`--background`, `--foreground`)
- Font variables: `--font-geist-sans` and `--font-geist-mono` configured via Next.js font optimization
- Dark mode uses `prefers-color-scheme` media query with CSS variable overrides
- Prefer utility classes over custom CSS

### React Patterns
- **React Compiler is enabled** - write idiomatic React, avoid manual memoization unless profiling shows it's needed
- Use Server Components by default (Next.js App Router convention)
- Add `"use client"` directive only when needed (state, effects, browser APIs)
- Prefer composition over prop drilling

### Font Loading
- Geist Sans and Geist Mono loaded via `next/font/google` in `layout.tsx`
- Font variables applied to `<body>` with template literals: `` `${geistSans.variable} ${geistMono.variable}` ``

## Important Configuration Notes

### Next.js Configuration
- React Compiler enabled in `next.config.ts` - this optimizes re-renders automatically
- Using TypeScript for config files (`.ts` not `.js`)

### TypeScript Configuration
- Module resolution: `bundler` (modern Next.js default)
- JSX: `react-jsx` (new JSX transform)
- Path alias `@/*` maps to `./src/*`

### Biome Configuration
- Indent: 2 spaces
- VCS integration enabled with Git
- Ignores: `node_modules`, `.next`, `dist`, `build`
- Domains: Next.js and React rules enabled
- Auto-organize imports on save via `assist.actions.source.organizeImports`

## When Adding New Features

1. **Components**: Place in `src/app/` (colocated with routes) or create `src/components/` for shared components
2. **API Routes**: Use `src/app/api/` with route handlers
3. **Styling**: Continue using Tailwind utilities; extend design tokens in `globals.css` `@theme` block if needed
4. **Types**: Create `src/types/` for shared type definitions
5. **Utilities**: Create `src/lib/` or `src/utils/` for helper functions

## Testing & Validation

- Always run `bun run lint` to catch issues before committing
- Run tests with `bun test` before committing
- Check dark mode behavior (defined via CSS variables in `globals.css`)
- Verify type safety with `tsc --noEmit` (already part of build process)

## Deployment

- Optimized for **Vercel** deployment (see README.md)
- Static assets go in `public/` directory
- Environment variables should use `NEXT_PUBLIC_` prefix for client-side access

---

# Game Engine Domain Model

## Architecture Overview

Echomancy is a **Trading Card Game (TCG)** engine inspired by Magic: The Gathering. The domain model follows a **CQRS-inspired pattern** where:

- `Game.apply(action)` is the single entry point for all state mutations
- Actions are discriminated union types (`PLAY_LAND`, `CAST_SPELL`, etc.)
- State is immutable from the outside, mutable internally via action handlers

```
┌─────────────────────────────────────────────────────────────┐
│                        Game (Aggregate)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │ PlayerState │    │    Stack    │    │ CreatureState│     │
│  │  (per player)│   │ (shared)    │    │  (per creature)    │
│  │  - hand      │    │  - spells   │    │  - isTapped  │     │
│  │  - battlefield│   │  - abilities│    │  - isAttacking│    │
│  │  - graveyard │    └─────────────┘    │  - hasAttacked│    │
│  └─────────────┘                        └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Core Domain Files

| File | Purpose |
|------|---------|
| `Game.ts` | Main aggregate - all game logic flows through here |
| `CardDefinition.ts` | Card templates (types, effects, abilities) |
| `CardInstance.ts` | Runtime card instances (instanceId, ownerId) |
| `Effect.ts` | Interface for spell/ability effects |
| `EffectContext.ts` | Context passed to effect resolution |
| `Zone.ts` | Container for cards (hand, battlefield, graveyard) |
| `Steps.ts` | Turn phases (UNTAP, FIRST_MAIN, COMBAT, etc.) |
| `StepMachine.ts` | State machine for turn progression |
| `GameErrors.ts` | Domain-specific error types |

## Key Patterns

### 1. Action Pattern (CQRS-style)

All game mutations go through `game.apply(action)`:

```typescript
// Playing a land
game.apply({ type: "PLAY_LAND", playerId: "p1", cardId: "land-1" })

// Casting a spell with targets
game.apply({
  type: "CAST_SPELL",
  playerId: "p1",
  cardId: "spell-1",
  targets: [{ kind: "PLAYER", playerId: "p2" }]
})

// Passing priority (for stack resolution)
game.apply({ type: "PASS_PRIORITY", playerId: "p1" })
```

### 2. Effect Interface

Effects implement a simple `resolve` interface:

```typescript
interface Effect {
  resolve(game: Game, context: EffectContext): void
}

// Example: DrawCardsEffect
class DrawCardsEffect implements Effect {
  constructor(private amount: number) {}

  resolve(game: Game, context: EffectContext): void {
    game.drawCards(context.controllerId, this.amount)
  }
}
```

### 3. Stack Resolution

Spells and abilities go on the stack and resolve LIFO:

```typescript
// 1. Cast spell → goes on stack
// 2. Opponent gets priority → can respond
// 3. Both players pass → top of stack resolves
// 4. Repeat until stack is empty
```

### 4. `enterBattlefield()` - Single Entry Point

**CRITICAL**: All permanents MUST enter via `game.enterBattlefield()`. This ensures:
- Creature state is initialized
- ETB (Enter The Battlefield) effects trigger
- Consistent behavior across all code paths

```typescript
// ✅ CORRECT - Using enterBattlefield
game.enterBattlefield(permanent, controllerId)

// ❌ WRONG - Direct manipulation
playerState.battlefield.cards.push(permanent) // Missing ETB triggers!
```

## Card Types

| Type | Stack? | Battlefield? | Notes |
|------|--------|--------------|-------|
| `CREATURE` | Yes | Yes | Has power/toughness, can attack |
| `INSTANT` | Yes | No | Resolves and goes to graveyard |
| `SORCERY` | Yes | No | Main phase only |
| `ARTIFACT` | Yes | Yes | Permanent |
| `ENCHANTMENT` | Yes | Yes | Permanent |
| `LAND` | No | Yes | One per turn, doesn't use stack |

## Game Steps (Turn Structure)

```
UNTAP → UPKEEP → DRAW → FIRST_MAIN →
BEGINNING_OF_COMBAT → DECLARE_ATTACKERS → DECLARE_BLOCKERS →
COMBAT_DAMAGE → END_OF_COMBAT → SECOND_MAIN →
END_STEP → CLEANUP → (next player's UNTAP)
```

---

# Testing Patterns

## Test File Location

All tests are in `src/echomancy/domainmodel/game/__tests__/`:

| Test File | Coverage |
|-----------|----------|
| `Game.test.ts` | Basic game setup and turn structure |
| `Game.creatures.test.ts` | Creature combat and state |
| `Game.castSpell.test.ts` | Spell casting and stack |
| `Game.effects.test.ts` | Effect resolution |
| `Game.etbTriggers.test.ts` | Enter-the-battlefield effects |
| `Game.activatedAbilities.test.ts` | Tap abilities, costs |
| `Game.targets.test.ts` | Target validation |

## Test Helpers (`helpers.ts`)

**Always use these helpers** instead of manual setup:

```typescript
import {
  createStartedGame,
  advanceToStep,
  createTestCreature,
  addCreatureToBattlefield,
  castSpellInMainPhase,
  resolveStack,
} from "./helpers"

describe("my test", () => {
  it("should work", () => {
    // Setup game with 2 players
    const { game, player1, player2 } = createStartedGame()

    // Add creature to battlefield (uses enterBattlefield internally!)
    const creature = createTestCreature(player1.id, "my-creature")
    addCreatureToBattlefield(game, player1.id, creature)

    // Advance to specific step
    advanceToStep(game, Step.DECLARE_ATTACKERS)

    // Cast and resolve a spell
    castSpellInMainPhase(game, player1.id)
    resolveStack(game, player2.id, player1.id)
  })
})
```

## Key Helpers Reference

| Helper | Purpose |
|--------|---------|
| `createStartedGame()` | Creates game with 2 players, ready to play |
| `advanceToStep(game, step)` | Fast-forward to a specific step |
| `createTestCreature(ownerId, id?)` | Creates a basic creature CardInstance |
| `createTestSpell(ownerId, id?)` | Creates a basic instant CardInstance |
| `addCreatureToBattlefield(game, playerId, creature)` | Adds creature via `enterBattlefield` |
| `addSpellToHand(game, playerId, spell)` | Puts spell in player's hand |
| `castSpellInMainPhase(game, playerId)` | Creates and casts a spell |
| `resolveStack(game, opponent, controller)` | Both players pass priority |
| `setupCreatureInCombat(game, playerId)` | Full setup for combat testing |

---

# Common Pitfalls

## ❌ Don't: Bypass `enterBattlefield()`

```typescript
// WRONG - Breaks ETB triggers and creature state
playerState.battlefield.cards.push(creature)
game.initializeCreatureStateIfNeeded(creature)

// RIGHT - Single source of truth
game.enterBattlefield(creature, playerId)
```

## ❌ Don't: Forget Stack Resolution

```typescript
// WRONG - Spell never resolves
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
// ...test assertions on unresolved state

// RIGHT - Resolve the stack first
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
resolveStack(game, opponentId, playerId)
// ...now test the resolved state
```

## ❌ Don't: Use Raw IDs in Tests

```typescript
// WRONG - Magic strings
const creature = { instanceId: "c1", ... }
game.apply({ type: "DECLARE_ATTACKER", creatureId: "c1" })

// RIGHT - Reference the object
const creature = createTestCreature(playerId)
addCreatureToBattlefield(game, playerId, creature)
game.apply({ type: "DECLARE_ATTACKER", creatureId: creature.instanceId })
```

## ❌ Don't: Ignore Action Validation

```typescript
// WRONG - Assuming actions always succeed
game.apply({ type: "PLAY_LAND", playerId, cardId })

// RIGHT - Use expect().toThrow() for invalid actions
expect(() => {
  game.apply({ type: "PLAY_LAND", playerId, cardId })
}).toThrow(LandLimitExceededError)
```

---

# Architectural Decisions

## Why `ts-pattern` for Action Matching?

The `match().with().exhaustive()` pattern ensures:
- All action types are handled
- TypeScript compiler catches missing cases
- Clean, declarative action routing

## Why No Mana System Yet?

MVP focuses on core mechanics. Mana will be added when:
- Card costs need implementation
- Mana abilities (lands tap for mana)
- Color system is designed

## Why Immediate ETB Resolution?

Current MVP executes ETB effects immediately, not as stack triggers. This simplifies:
- Testing and debugging
- State predictability
- Initial implementation

Future: ETB triggers will use the stack for proper response windows.

---

# Quick Reference

## Common Commands

```bash
bun test                          # Run all tests
bun test Game.creatures           # Run specific test file
bun test --watch                  # Watch mode
bun run lint                      # Check code style
bun run format                    # Auto-format code
```

## Import Paths

```typescript
// Game and types
import { Game, AllowedAction, CreatureState } from "@/echomancy/domainmodel/game/Game"
import { Step } from "@/echomancy/domainmodel/game/Steps"
import { Player } from "@/echomancy/domainmodel/game/Player"

// Cards
import type { CardDefinition, CardType } from "@/echomancy/domainmodel/cards/CardDefinition"
import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"

// Effects
import type { Effect } from "@/echomancy/domainmodel/effects/Effect"
import type { EffectContext } from "@/echomancy/domainmodel/effects/EffectContext"

// Errors
import { CardNotFoundInHandError, PermanentNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
```

## Action Types Quick Reference

```typescript
{ type: "ADVANCE_STEP", playerId }
{ type: "END_TURN", playerId }
{ type: "PLAY_LAND", playerId, cardId }
{ type: "CAST_SPELL", playerId, cardId, targets }
{ type: "PASS_PRIORITY", playerId }
{ type: "DECLARE_ATTACKER", playerId, creatureId }
{ type: "ACTIVATE_ABILITY", playerId, permanentId }
```

## Target Types

```typescript
{ kind: "PLAYER", playerId: string }
// Future: { kind: "PERMANENT", permanentId: string }
// Future: { kind: "CARD_IN_GRAVEYARD", cardId: string }
```
