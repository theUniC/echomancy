---
name: software-architecture
description: Guide for quality focused software architecture. This skill should be used when users want to write code, design architecture, analyze code, in any case that relates to software development. (project)
---

# Software Architecture Development Skill

This skill provides guidance for quality focused software development and architecture based on Clean Architecture and Domain Driven Design principles.

## When to Use

Invoke this skill when:
- Designing new features or systems
- Reviewing code architecture
- Refactoring existing code
- Making decisions about patterns or structure

## Process

### 1. Understand Context

Before making architectural decisions:
- Read `AGENTS.md` for project rules
- Read relevant `docs/*.md` for existing architecture
- Examine similar existing code patterns
- Identify the bounded context

### 2. Apply Architecture Checklist

For each piece of code, verify:

| Check | Question |
|-------|----------|
| Domain isolation | Is business logic separate from infrastructure? |
| Single responsibility | Does each module do one thing? |
| Dependency direction | Do dependencies point inward (toward domain)? |
| Naming | Are names domain-specific, not generic? |
| Library-first | Does a library solve this already? |

### 3. Echomancy-Specific Patterns

**Game state mutations:**
```typescript
// CORRECT - Use apply()
game.apply({ type: "PLAY_LAND", playerId, cardId })

// WRONG - Direct mutation
playerState.hand.cards.splice(index, 1)
```

**Permanents entering battlefield:**
```typescript
// CORRECT - Use enterBattlefield()
game.enterBattlefield(permanent, controllerId)

// WRONG - Direct push
playerState.battlefield.cards.push(permanent)
```

**Action matching:**
```typescript
// Use ts-pattern with exhaustive()
match(action)
  .with({ type: "CAST_SPELL" }, handleCast)
  .with({ type: "PLAY_LAND" }, handleLand)
  .exhaustive()
```

**New effects:** Place in `src/echomancy/domainmodel/effects/impl/`

**Domain errors:** Extend classes in `GameErrors.ts`

## Code Style Rules

### General Principles

- **Early return pattern**: Use early returns over nested conditions
- Avoid code duplication through reusable functions
- Decompose long (>80 lines) components into smaller ones
- Use arrow functions instead of function declarations
- Max 3 levels of nesting
- Functions under 50 lines, files under 200 lines

### Library-First Approach

**ALWAYS search for existing solutions before writing custom code:**
- Check npm for existing libraries
- Evaluate existing services/SaaS solutions
- Consider third-party APIs

**When custom code IS justified:**
- Specific business logic unique to the domain
- Performance-critical paths with special requirements
- Security-sensitive code requiring full control
- When existing solutions don't meet requirements

### Naming Conventions

| Avoid | Use Instead |
|-------|-------------|
| `utils`, `helpers`, `common` | Domain-specific names |
| `misc.js`, `shared.js` | `OrderCalculator`, `ManaPool` |
| Generic function names | Descriptive domain terms |

### Anti-Patterns to Avoid

- **NIH Syndrome**: Don't build what exists (auth, state management, validation)
- **Mixed concerns**: Keep business logic out of UI components
- **Generic dumping grounds**: No `utils.js` with 50 unrelated functions
- **Deep nesting**: Refactor if >3 levels deep

## TypeScript Guidelines

- Use `type` over `interface`
- Use `@/*` path alias for imports from `src/`
- No `any` types (strict mode is enabled)
- Proper error handling with typed catch blocks

## React Guidelines (when applicable)

- Server Components by default
- `"use client"` only when state/effects/browser APIs needed
- No manual `useMemo`/`useCallback` (React Compiler handles it)

## Verification

Before completing architectural work:
- [ ] Business logic is isolated from infrastructure
- [ ] No generic naming (`utils`, `helpers`)
- [ ] Dependencies point toward domain
- [ ] Existing libraries used where appropriate
- [ ] Code follows project patterns from `AGENTS.md`
- [ ] Tests exist for new functionality
