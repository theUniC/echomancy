# Agent Instructions for Echomancy

TCG engine built with Next.js 16, React 19, TypeScript, and Tailwind CSS v4.

## First Action

Read `docs/README.md` before doing anything else. The documentation explains architectural decisions and constraints you must follow.

## Critical Rules

1. **English only** - All code, docs, commits, tests, error messages. No exceptions.
2. **Read docs before coding** - Read relevant `docs/*.md` files before any implementation.
3. **Update docs after coding** - Keep documentation synchronized with the engine.
4. **Prefer Bun** - Use `bun install`, `bun run test`. Only use npm if Bun fails.
5. **Ask before commit/push** - Always get explicit user confirmation first.
6. **Use vitest** - Import from `"vitest"`, never `"bun:test"` (breaks CI).

## Backlog Workflow

Specs location: `docs/specs/` with three folders: `backlog/` → `active/` → `done/`

**Find next task:**
1. Open `docs/specs/BACKLOG.md`
2. Find first item with status `TODO`
3. Spec file is in `docs/specs/backlog/` (e.g., `B1-04` → `B1-04-summoning-sickness.md`)

**Start work:**
1. Update `BACKLOG.md`: change status to `IN PROGRESS`
2. Move spec from `backlog/` to `active/`

**Complete work:**
1. Update `BACKLOG.md`: change status to `DONE`, unblock dependent items
2. Move spec from `active/` to `done/`

## Task Workflow

1. **Read docs** - Read `docs/README.md` and relevant files
2. **Understand** - Read relevant source files before writing code
3. **Check patterns** - Look at existing similar code/tests
4. **Implement** - Write the code
5. **Test** - Run `bun run test`
6. **Lint** - Run `bun run lint && bun run format`
7. **Update docs** - If you changed functionality, update relevant docs
8. **Commit** - Only if steps 5-6 pass

## Commands

```bash
bun run dev          # Dev server at localhost:3000
bun run build        # Production build
bun run test         # Run all tests
bun run test <name>  # Run tests matching name
bun run lint         # Check code style
bun run format       # Auto-format code
```

## File Locations

| What | Where |
|------|-------|
| Documentation | `docs/` |
| MTG Rules Reference | `docs/reference/MagicCompRules-*.txt` |
| Specifications | `docs/specs/` |
| Game engine core | `src/echomancy/domainmodel/game/` |
| Test helpers | `src/echomancy/domainmodel/game/__tests__/helpers.ts` |
| Specialized agents | `.claude/agents/` |
| Reusable skills | `.claude/skills/` |
