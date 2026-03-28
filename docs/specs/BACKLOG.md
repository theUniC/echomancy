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
| Core Engine | Complete | Rust, 630+ tests, DDD architecture |
| Bevy UI | Basic | Play lands, pass priority, end turn |
| Playable Game | **No** | Can't cast spells, no combat UI, no game end |
| Tech Stack | Rust / Bevy 0.18 | Single native binary |

### Goal: v0.1.0 — Minimum Playable Game

A real game of Magic where two players can play lands, cast creatures, attack, block, and someone wins.

---

## Backlog (Prioritized)

Work items in order of implementation. **Always take the first `TODO` item.**

### Legend
- `DONE` - Completed and in `docs/specs/done/`
- `IN PROGRESS` - Currently being worked on, spec in `docs/specs/active/`
- `TODO` - Ready to implement, spec in `docs/specs/backlog/`
- `BLOCKED` - Cannot start until dependency is done

### Milestone: v0.1.0 — Playable Game

These items, in order, make the game actually playable.

| # | Description | Status | Dependency |
|---|-------------|--------|------------|
| A | Mana abilities (tap land → add mana to pool) | TODO | - |
| B | Spell resolution (creature spells enter battlefield) | TODO | A |
| C | Cast spell UI (click creature in hand → pay mana → stack) | BLOCKED | A, B |
| D | Mana pool display in HUD | BLOCKED | A |
| E | Combat UI (declare attackers + blockers) | BLOCKED | C |
| F | Game end display ("You Win" / "You Lose") | TODO | - |
| G | Two-player hotseat (alternate perspectives each turn) | BLOCKED | E, F |

### Future: Combat & Interaction UI

| # | Description | Status | Dependency |
|---|-------------|--------|------------|
| 8 | UI: Spell casting with targets | TODO | v0.1.0 |
| 9 | UI: Stack display | TODO | - |
| 10 | UI: Activated abilities | TODO | v0.1.0 |
| 11 | UI: Graveyard viewer | TODO | - |
| 12 | UI: Exile zone | TODO | - |

---

## Completed Work

### Rust Migration (Done)

Full migration from TypeScript/Next.js to Rust/Bevy completed March 2026.

### Core Engine (Done)

- Turns and phases (12 steps, 5 phases)
- Priority and stack (LIFO resolution, priority passing)
- Zones: Hand, Battlefield, Graveyard, Library
- Library zone + drawCards()
- Game setup (deck loading, shuffle, draw 7)
- Win/lose conditions (life total, empty library, draws)
- DDD Architecture: command handlers, specifications, services, value objects
- ETB triggers (execute immediately, not on stack)
- Combat: declare attackers/blockers, damage, cleanup
- Mana pool (add/spend/clear)
- Spell timing validation (instant vs sorcery) + Flash keyword
- Mana cost payment with auto-pay algorithm
- Costs: TapSelfCost, SacrificeSelfCost, ManaCost
- Static keywords: Flying, Reach, Vigilance, Haste
- Summoning sickness
- +1/+1 counters and P/T calculation
- PermanentState - unified state for all permanents
- GameSnapshot - player-relative filtered view
- Application layer (CQRS commands/queries)

### Bevy UI (Done)

- Battlefield display (player + opponent zones)
- Hand display with card rendering (name, type, P/T, colored borders)
- Play land interaction (click playable land → moves to battlefield)
- HUD panel (turn/step, priority indicator, life totals, hand/graveyard counts)
- Priority controls (Pass Priority, End Turn buttons)
- Error message display
- Tapped card rotation

---

## Known Limitations (MVP)

Intentional simplifications:

| Limitation | Reason |
|------------|--------|
| Triggers execute immediately (not on stack) | Proper trigger stacking is complex |
| 1 blocker per attacker | Simplifies damage assignment |
| Only player targets | Creature/permanent targeting requires more work |
| Auto-tap lands for mana | No manual mana ability activation yet |
| Mana empties only at Cleanup | MTG says each phase, simplified |
| No 7-layer system | Only needed for continuous effects |
| resolve_ability is no-op | Activated abilities don't execute effects yet |
| Single player perspective | Both players controlled from same view |

---

## Post-MVP Features

### Combat Keywords
- First Strike / Double Strike
- Trample
- Deathtouch
- Indestructible

### Counters
- -1/-1 counters
- Poison counters
- Loyalty counters

### Mana & Costs
- X costs, hybrid mana, Phyrexian mana
- Alternative costs, cost reductions
- Snow mana, conditional mana

### Effects & Abilities
- Duration tracking ("until end of turn")
- Modal effects ("Choose one")
- Prevention/replacement effects
- Token creation

### Targeting
- Creature/permanent targeting
- Target validation on resolution

### Advanced Rules
- Multiple blockers + damage assignment order
- Continuous effects / lords
- 7-layer system
- Hexproof / Shroud / Protection
- Full state-based actions (legend rule, etc.)

### Game Modes
- Mulligan system
- Deck builder
- Matchmaking / networking
- Replays

---

## Design Philosophy

Echomancy prioritizes:
1. **Correctness over performance** - Rules must be right
2. **Explicitness over convenience** - No hidden magic
3. **Testability over flexibility** - Everything must be testable
4. **Type safety** - Rust's type system enforces invariants at compile time
5. **Transparency** - Open source, honest about limitations

---

## Updating This Document

When completing work:
1. Change status from `IN PROGRESS` to `DONE`
2. Update any `BLOCKED` items that are now unblocked to `TODO`
3. Add completed item to "Completed Work" section if significant
