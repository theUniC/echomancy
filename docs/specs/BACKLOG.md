# Echomancy — Backlog & Project Status

This is the **single source of truth** for project status and prioritized work.

---

## How to Use This Document

1. **Find next work**: Look at the Backlog table below - take the first item with status `TODO`
2. **Start work**: Change status to `IN PROGRESS`, move spec to `docs/specs/active/`
3. **Finish work**: Change status to `DONE`, move spec to `docs/specs/done/`
4. **Spec details**: Each backlog item has a detailed spec in `docs/specs/backlog/`

---

## Project Status

| Area | Status | Notes |
|------|--------|-------|
| Core Engine | Partial | Missing: mana costs, win/lose, spell timing |
| UI | In Progress | Basic display done, interactions partially done |
| MVP Complete | No | Blocked by core engine gaps |

---

## Backlog (Prioritized)

Work items in order of implementation. **Always take the first `TODO` item.**

### Legend
- `DONE` - Completed and in `docs/specs/done/`
- `IN PROGRESS` - Currently being worked on, spec in `docs/specs/active/`
- `TODO` - Ready to implement, spec in `docs/specs/backlog/`
- `BLOCKED` - Cannot start until dependency is done

### Priority 0: Foundation (Blocking)

This refactor is foundational and should be done before other backlog items.

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 0 | B0-01 | Game.ts DDD Refactor (2,282 → ~600-800 lines) | DONE | - |

### Priority 1: Core Engine Fixes (Parallel)

These fix critical bugs and gaps. Can be done in parallel.

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 1 | B1-04 | Summoning sickness + Haste keyword | DONE | - |
| 2 | B1-05 | Spell timing (instant vs sorcery) + Flash | DONE | B0-01 ✓ |
| 3 | B1-06 | Mana cost payment (manaCost field, generic mana) | DONE | B0-01 (Phase 1-2) |

### Priority 2: Core Engine Foundation (Sequential)

These enable real game flow. Must be done in order.

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 4 | B1-01 | Library zone + drawCards() | DONE | - |
| 5 | B1-02 | Game setup (deck loading, shuffle, draw 7) | TODO | B1-01 ✓ |
| 6 | B1-03 | Win/lose conditions (life <= 0, empty library) | TODO | B1-01 ✓ |

### Priority 3: Combat UI (Sequential)

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 7 | 09 | UI: Declare attackers | TODO | B1-04 ✓ |
| 8 | 10 | UI: Declare blockers | BLOCKED | 09 |
| 9 | 11 | UI: Combat damage | BLOCKED | 10 |

### Priority 4: Spell & Game End UI

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 10 | 08 | UI: Spell casting with targets | TODO | B1-05 ✓, B1-06 ✓ |
| 11 | 14 | UI: Game end display | BLOCKED | B1-03 |

### Priority 5: Complementary UI

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 12 | 15 | UI: Stack display | TODO | - |
| 13 | 13 | UI: Activated abilities | TODO | B1-04 ✓ |

### Priority 6: Nice to Have

| # | Spec | Description | Status | Dependency |
|---|------|-------------|--------|------------|
| 14 | 12 | UI: Graveyard viewer | TODO | - |
| 15 | 16 | UI: Exile zone | TODO | - |

---

## Completed Work

### Core Engine (Done)

- Turns and phases (12 steps, 5 phases)
- Priority and stack (LIFO resolution, priority passing)
- Zones: Hand, Battlefield, Graveyard, Library
- Library zone + drawCards() (B1-01)
- Game.ts DDD Refactor - Specifications, Value Objects, Services (B0-01)
- ETB triggers (execute immediately, not on stack)
- Combat: declare attackers/blockers, damage, cleanup
- Mana pool (add/spend/clear) - exists but not used for spell costs
- Spell timing validation (instant vs sorcery) + Flash keyword (B1-05)
- Mana cost payment with auto-pay algorithm (B1-06)
- Costs: TapSelfCost, SacrificeSelfCost, ManaCost (not wired to spells)
- Static keywords: Flying, Reach, Vigilance (consultative)
- +1/+1 counters
- Creature P/T calculation

### UI (Done)

- Debug console (Phase 0)
- Route & data pipeline (Phase 1a)
- Basic game info - turn, phase, life (Phase 1b)
- Battlefield display with PixiJS (Phase 1c)
- Hand display (Phase 1d)
- Graveyard count (Phase 1e)
- Play land interaction (Phase 2)
- Priority controls - Pass/End Turn (Phase 2)

---

## Known Limitations (MVP)

These are intentional simplifications for MVP:

| Limitation | Reason |
|------------|--------|
| Triggers execute immediately (not on stack) | Complexity - proper trigger stacking is complex |
| 1 blocker per attacker | Simplifies damage assignment |
| Only player targets | Creature/permanent targeting requires more work |
| No mana abilities | Auto-tap lands for now |
| Mana empties only at Cleanup | MTG rule says each phase, but simplified |
| No 7-layer system | Only needed for continuous effects (lords, etc.) |

---

## Post-MVP Features

Features explicitly deferred beyond MVP:

### Combat Keywords
- First Strike / Double Strike
- Trample
- Deathtouch
- Indestructible

### Counters
- -1/-1 counters
- Poison counters
- Loyalty counters

### Advanced Rules
- Multiple blockers
- Replacement effects
- Continuous effects / lords
- 7-layer system
- Hexproof / Shroud / Protection

### Game Modes
- Mulligan system
- Deck builder
- Matchmaking
- Replays

---

## Design Philosophy

Echomancy prioritizes:
1. **Correctness over performance** - Rules must be right
2. **Explicitness over convenience** - No hidden magic
3. **Testability over flexibility** - Everything must be testable
4. **Type safety** - TypeScript strict mode everywhere

---

## Updating This Document

When completing work:
1. Change status from `IN PROGRESS` to `DONE`
2. Update any `BLOCKED` items that are now unblocked to `TODO`
3. Add completed item to "Completed Work" section if significant
