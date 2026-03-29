# Echomancy — Backlog & Project Status

This is the **single source of truth** for project status and prioritized work.

---

## How to Use This Document

1. **Find next work**: Look at the Backlog table below - take the first `TODO` item
2. **Start work**: Change status to `IN PROGRESS`, move spec to `docs/specs/active/`
3. **Finish work**: Change status to `DONE`, move spec to `docs/specs/done/`

---

## Project Status

| Area | Status | Notes |
|------|--------|-------|
| Core Engine | Complete | Rust, 665+ tests, DDD architecture |
| Bevy UI | Functional | Play lands, cast creatures, combat, game end |
| Rules Engine | PoC done | CLIPS 6.4.2 via C FFI, architecture designed |
| Playable Game | Partial | Lands + creatures + combat work. No spell effects, no AI, no networking |
| Tech Stack | Rust / Bevy 0.18 / CLIPS 6.4.2 | Single native binary |

---

## Backlog (Prioritized)

### Legend
- `DONE` — Completed
- `IN PROGRESS` — Currently being worked on
- `TODO` — Ready to implement
- `BLOCKED` — Cannot start until dependency is done

### Phase 1: CLIPS Integration (extends existing engine)

See `docs/architecture-clips-integration.md` for full design spec.

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| M1 | ClipsEngine safe wrapper + router + tests | DONE | clips-sys PoC ✅ | `infrastructure/clips/`, `RulesEngine` trait |
| M2 | Bridge: serialize Game state → CLIPS facts | DONE | M1 | Full reset cycle, deftemplates |
| M3 | Connect `resolve_spell()` to CLIPS | DONE | M1, M2 | Minimal change to existing internals.rs |
| M4 | Connect triggered abilities to CLIPS | DONE | M3 | Wire `execute_triggered_abilities()` |
| M5 | Write .clp rules for existing cards (Bear, Goblin, etc.) | DONE | M3 | Core rules + card rules |
| M6 | MTGJSON card data loader | DONE | M5 | CardDefinition from JSON, load at startup |

### Phase 2: Expanding Card Coverage

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| P1 | Target selection system (creature/player targeting) | DONE | M3 | Required for damage spells, removal |
| P2 | Instant-speed casting (during combat, opponent's turn) | TODO | M3 | Priority before combat damage |
| P3 | More card types: Enchantments, Artifacts | TODO | M4 | Static abilities, continuous effects |
| P4 | Continuous effects / 7-layer system in CLIPS | TODO | P3 | Lords, enchantments, buff/debuff |
| P5 | Combat keywords (First Strike, Trample, Deathtouch, Lifelink) | DONE | M3 | Domain model, no CLIPS needed |

### Phase 3: Game Experience

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| G1 | Basic AI opponent (plays lands, casts creatures, attacks) | TODO | M5 | Single-player mode |
| G2 | MTGJSON Oracle text → .clp auto-generation | TODO | M6 | ~25-30% of cards auto-generated |
| G3 | P2P networking | TODO | G1 | Real multiplayer |
| G4 | Deck builder | TODO | M6, G6 | Choose cards from MTGJSON catalog |
| G5 | Mulligan system | TODO | - | Vancouver/London mulligan |
| G6 | Build-time card catalog pre-compilation | TODO | M6 | `build.rs` or CLI to convert AtomicCards.json → binary format, avoid parsing 100MB JSON at every startup |

### Phase 4: UI Polish

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| U1 | Stack display (show pending spells/abilities) | TODO | M3 | |
| U2 | Graveyard viewer | TODO | - | |
| U3 | Exile zone | TODO | - | |
| U4 | Card detail view (hover/click for full text) | TODO | - | |
| U5 | Animations (card movement, damage, phase transitions) | TODO | - | |
| U6 | Hotseat transition screen | TODO | - | "Pass to Player 2" overlay |

---

## Completed Work

### Rust Migration (March 2026)

Full migration from TypeScript/Next.js to Rust/Bevy.

### Core Engine

- Turns and phases (12 steps, 5 phases)
- Priority and stack (LIFO resolution, priority passing)
- Zones: Hand, Battlefield, Graveyard, Library
- Game setup (deck loading, shuffle, draw 7)
- Win/lose conditions (life total, empty library, draws)
- DDD Architecture: command handlers, specifications, services, value objects
- ETB triggers (detect, not yet execute via CLIPS)
- Combat: declare attackers/blockers, damage, cleanup
- Mana pool + mana abilities (tap land → add mana)
- Spell timing validation (instant vs sorcery) + Flash keyword
- Mana cost payment with auto-pay algorithm
- Static keywords: Flying, Reach, Vigilance, Haste
- Summoning sickness
- +1/+1 counters and P/T calculation

### Bevy UI

- Battlefield display (player + opponent zones)
- Hand display (name, type, P/T, colored borders)
- Play land interaction (click → moves to battlefield)
- Cast creature interaction (click → pay mana → resolves to battlefield)
- Tap land for mana (click → land taps, mana added)
- Combat UI (declare attackers with red border, blockers with blue)
- HUD panel (turn/step, priority, life totals, mana pool, hand/graveyard counts)
- Pass Priority / End Turn buttons
- Active player switching (perspective follows current player)
- Game end overlay (YOU WIN / YOU LOSE / DRAW)
- Error message display with humanized player names

### Architecture Refactoring

- Split game/mod.rs (1,939 → 746 lines)
- Moved auto-advance/resolve to core
- Moved game_state_export to infrastructure
- Removed unused application layer (CQRS/repository)
- Split plugins/game.rs (1,085 → 3 files)
- CreatureSubState extracted to own file
- Thin mutators for Game internal state

### CLIPS PoC (March 2026)

- clips-sys crate: CLIPS 6.4.2 compiled from C source
- SBA rules in CLIPS: correct results, ~7-11µs per cycle
- Architecture design document complete

---

## Known Limitations

| Limitation | Reason | Resolved by |
|------------|--------|-------------|
| Spell effects are no-ops | No rules engine connected yet | M3 |
| Triggered abilities don't execute | No CLIPS integration yet | M4 |
| No continuous effects / layers | Not implemented | P4 |
| 1 blocker per attacker | MVP simplification | Future |
| Only player targets | No target selection system | P1 |
| No priority before combat damage | Auto-advances through combat | P2 |
| No AI opponent | Not implemented | G1 |
| No networking | Not implemented | G3 |
| 5 hardcoded cards | No card database | M6 |

---

## Design Philosophy

Echomancy prioritizes:
1. **Correctness over performance** — Rules must be right
2. **Explicitness over convenience** — No hidden magic
3. **Testability over flexibility** — Everything must be testable
4. **Type safety** — Rust's type system enforces invariants at compile time
5. **Transparency** — Open source, honest about limitations
6. **Honesty** — If we can't implement something correctly, we say so

---

## Updating This Document

When completing work:
1. Change status from `IN PROGRESS` to `DONE`
2. Update any `BLOCKED` items that are now unblocked to `TODO`
3. Add completed item to "Completed Work" section
