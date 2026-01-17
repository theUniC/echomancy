# Agent Instructions for Echomancy

**Echomancy** is a Trading Card Game (TCG) engine built with Next.js 16, React 19, TypeScript, and Tailwind CSS v4.

---

## Finding the Next Task

**Always check `docs/specs/BACKLOG.md` first** to find the next prioritized work item.

1. Open `docs/specs/BACKLOG.md`
2. Find the first item with status `TODO` in the backlog tables
3. The spec file is in `docs/specs/backlog/` with the spec ID (e.g., `B1-04` → `B1-04-summoning-sickness.md`)
4. When starting work: change status to `IN PROGRESS` and move spec to `docs/specs/active/`
5. When done: change status to `DONE` and move spec to `docs/specs/done/`

> **Why**: The backlog is prioritized. Working on items out of order may create dependency issues or waste effort on lower-priority features.

---

## Task Workflow

When starting any task, follow this order:

1. **Read docs first** - Read all files in `docs/` before doing anything else
2. **Understand** - Read relevant source files before writing code
3. **Check patterns** - Look at existing similar code/tests
4. **Implement** - Write the code
5. **Test** - Run `bun test`
6. **Lint** - Run `bun run lint && bun run format`
7. **Update docs** - If you added/changed functionality, update relevant docs
8. **Commit** - Only if steps 5-6 pass

---

## P0: Never Violate

These rules exist to prevent silent bugs that pass tests but break game logic.

### All content must be in English

Everything in this project MUST be written in English:

- Code (variables, functions, comments)
- Documentation (docs/, docs/specs/, README, BACKLOG)
- Commit messages and PR descriptions
- Test descriptions and assertions
- Error messages and logs

No exceptions. This ensures consistency and accessibility for all contributors.

### Read documentation before any task

Before writing any code, read relevant documentation in `docs/`:

1. Start with `docs/README.md` for the documentation index
2. Read files relevant to your task (architecture, abilities, effects, etc.)
3. Check `docs/reference/` for MTG Comprehensive Rules when validating game mechanics

> **Why**: The documentation explains architectural decisions and constraints. Without understanding these, you will likely violate core principles.

### Update documentation when adding features

When you implement new functionality:

1. Check if it affects concepts described in `docs/`
2. Update the relevant documentation file
3. Keep explanations conceptual (no code blocks in docs)
4. Add new MVP limitations if applicable

> **Why**: Documentation must stay synchronized with the engine. Outdated docs cause future agents to make incorrect assumptions.

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

### Prefer Bun, npm only as fallback

```bash
# PREFERRED
bun install
bun test
bun run dev

# FALLBACK - only if bun install fails
npm install
```

> **Why**: Bun is faster and the primary package manager. Use npm only if Bun fails due to compatibility issues.

### API routes never import from domainmodel/

API routes can only import from `application/` and `infrastructure/`. If a Command or Query doesn't exist, create it first.

```typescript
// CORRECT
const handler = new StartGameCommandHandler(gameRepository)
handler.handle(new StartGameCommand(gameId, startingPlayerId))

// WRONG - direct domain access from API route
const game = gameRepository.byId(gameId)
game.start(startingPlayerId)  // Bypasses application layer
```

> **Why**: The application layer (`Commands/Queries`) is the only entry point to the domain. This ensures validation, error handling, and business logic are centralized. API routes are just HTTP adapters.

**Exceptions** (re-exported via `application/`):
- `application/errors.ts` - Domain errors for API error handling
- `application/types.ts` - `Actions` and `GameStateExport` for API contracts

These define input/output shapes without exposing domain logic.

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

### Always ask before committing or pushing

**MANDATORY**: Before executing any `git commit` or `git push` command, you MUST ask the user for explicit confirmation. Never commit or push changes without user approval.

> **Why**: The user needs full control over what goes into the repository. Unexpected commits can disrupt workflows, include incomplete work, or push changes the user wanted to review first.

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
| Starting without reading `docs/` | Read all documentation first |
| `battlefield.cards.push(...)` | Use `enterBattlefield()` |
| `hand.cards.splice(...)` | Use `game.apply()` with appropriate action |
| `new Game(...)` in tests | Use `createStartedGame()` |
| `npm install` without trying bun first | Try `bun install` first, npm only as fallback |
| Using `any` type | Find or create proper type |
| Skipping `resolveStack()` | Always resolve before asserting effects |
| Committing without `bun test` | Run tests first |
| Committing/pushing without asking user | Always ask for explicit confirmation first |
| Adding feature without updating docs | Update relevant `docs/*.md` file |
| API route importing from `domainmodel/` | Use Commands/Queries from `application/` |

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

## Specifications (docs/specs/)

All feature specifications live in `docs/specs/` with a backlog-based workflow.

### Folder Structure

```
docs/specs/
├── backlog/    # Specs waiting to be implemented (prioritized by number)
│   ├── 01-next-feature.md
│   ├── 02-another-feature.md
│   └── ...
├── active/     # Currently being implemented (1-2 specs max)
└── done/       # Completed specs (reference)
```

### Workflow

1. **New specs** -> `docs/specs/backlog/` with numeric prefix (01-, 02-, etc.)
2. **Start work** -> Move spec from `backlog/` to `active/`
3. **Finish work** -> Move spec from `active/` to `done/`

### Rules

- **Agents read only `docs/specs/active/`** - They implement what's there
- **Humans move specs** between folders to control what gets built
- **One spec at a time** - Keep `active/` small to avoid big bang implementations

### Naming conventions

- Numeric prefix for priority: `01-`, `02-`, etc.
- Use kebab-case: `01-combat-system.md`, `02-mana-pool.md`
- Be descriptive but concise

---

## Commands

```bash
bun run dev      # Dev server at localhost:3000
bun run build    # Production build
bun run test         # Run all tests
bun run test <name>  # Run tests matching name
bun run lint     # Check code style
bun run format   # Auto-format code
```

---

## File Locations

| What | Where |
|------|-------|
| Documentation | `docs/` (read this first!) |
| MTG Rules Reference | `docs/reference/MagicCompRules-*.txt` |
| Specifications | `docs/specs/` (features, architecture, mechanics) |
| Game engine core | `src/echomancy/domainmodel/game/` |
| Card/Effect types | `src/echomancy/domainmodel/cards/`, `effects/` |
| Test helpers | `src/echomancy/domainmodel/game/__tests__/helpers.ts` |
| All tests | `src/echomancy/domainmodel/game/__tests__/` |

---

## Claude Code Resources

| What | Where |
|------|-------|
| Specialized agents | `.claude/agents/` |
| Reusable skills | `.claude/skills/` (invoke with `/skill-name`) |
