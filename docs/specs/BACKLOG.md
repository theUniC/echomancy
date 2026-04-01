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
| Core Engine | Complete | Rust, 1000+ tests, DDD architecture |
| Bevy UI | Functional | Play lands, cast creatures/instants, combat, game end |
| Rules Engine | Integrated | CLIPS 6.4.2 via C FFI, spell effects work |
| Bot (P2) | Functional | Greedy bot: plays lands, casts, attacks. Replaces hotseat |
| Playable Game | Single-player | P1 vs bot. Instant-speed responses, priority flow, combat keywords |
| Tech Stack | Rust / Bevy 0.18 / CLIPS 6.4.2 | Single native binary |

---

## Backlog (Prioritized)

### Legend
- `DONE` — Completed
- `IN PROGRESS` — Currently being worked on
- `TODO` — Ready to implement
- `BLOCKED` — Cannot start until dependency is done

### Phase 1: CLIPS Integration ✅

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
| P2 | Instant-speed casting (during combat, opponent's turn) | DONE | M3 | Full MTG priority per CR 117 |
| P3 | More card types: Enchantments, Artifacts | DONE | M4 | Sol Ring (artifact mana), Wild Bounty (enchantment ETB) |
| P4 | Continuous effects (temporary P/T modification) | DONE | - | Giant Growth +3/+3, expires at Cleanup |
| P5 | Combat keywords (First Strike, Trample, Deathtouch, Lifelink) | DONE | M3 | Domain model, no CLIPS needed |

### Phase 3: Game Experience

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| G1 | Basic AI opponent (plays lands, casts creatures, attacks) | DONE | M5 | Greedy bot replaces hotseat, P1 perspective fixed |
| G2 | MTGJSON Oracle text → .clp auto-generation | TODO | M6 | ~25-30% of cards auto-generated |
| G3 | P2P networking | TODO | G1 | Real multiplayer |
| G4 | Deck builder | TODO | M6, G6 | Choose cards from MTGJSON catalog |
| G5 | Mulligan system | DONE | - | Vancouver mulligan with put-back |
| G6 | Build-time card catalog pre-compilation | TODO | M6 | `build.rs` or CLI to convert AtomicCards.json → binary format |

### Phase 4: UI Polish

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| U1 | Stack display (show pending spells/abilities) | DONE | M3 | Centered overlay, auto-hides when empty |
| U2 | Graveyard viewer | DONE | - | Clickable HUD labels toggle overlay |
| U3 | Exile zone (domain + viewer) | DONE | - | Full zone in domain + RulesAction::Exile + UI viewer |
| U4 | Card detail view (hover for full text) | DONE | - | Hover panel with name, type, cost, oracle text, P/T |
| U5 | Animations (card movement, damage, phase transitions) | TODO | - | |
| U6 | ~~Hotseat transition screen~~ | N/A | - | Superseded by G1 (bot replaces hotseat) |
| U7 | Battlefield overflow (horizontal scroll) | DONE | - | scroll_x on battlefield + hand zones |
| U8 | Multiple blocker assignment UI (N blockers → 1 attacker) | BLOCKED | C6 | Needs domain support first |
| U10 | Damage assignment UI (attacker chooses how to split damage among blockers) | BLOCKED | C6 | CR 510.1c — currently auto-assigned |
| U9 | Non-land artifacts tappable for mana | DONE | - | compute_tappable_lands includes any mana ability permanent |

### Phase 5: Critical Rules Fixes

Identified by MTG domain expert audit (April 2026). These are rules violations
that affect correctness of every game.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| C1 | Mana pools must empty at each step, not just Cleanup | DONE | 106.4 | Low | - | Cleared in perform_step_advance |
| C2 | Double Strike keyword | DONE | 702.4 | Medium | - | Deals damage in both first-strike and normal combat damage steps |
| C3 | Counterspells / target spells on the stack | DONE | 114.1 | High | R1 ✅ | Cancel card + RulesAction::CounterSpell + CLIPS rule |
| C4 | Verify target legality at resolution ("fizzle") | DONE | 608.2b | Medium | - | Spell goes to GY without effect if all targets illegal |
| C5 | SBA loop must repeat until no more actions taken | DONE | 704.3 | Low | - | Loops up to 20 iterations |
| C5b | SBA infinite loop should declare draw (CR 104.4b) | DONE | 104.4b | Low | C5 | Declares Draw with InfiniteLoop reason at cap=20 |
| C6 | Multiple blockers per attacker + damage ordering | DONE | 509.1a | High | - | Auto-assign damage (smallest first), UI pending (U8/U10) |

### Phase 6: Major Missing Mechanics

Core MTG systems not yet implemented, ordered by dependency chain.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| R1 | Expand target system (artifact, enchantment, permanent, stack spell) | DONE | 114-115 | High | - | Unblocks counterspells (C3) and most removal |
| R2 | Card subtypes (creature types, land types, etc.) | TODO | 205.3 | Medium | - | Needed for tribal, fetching, many interactions |
| R3 | Legendary rule (legend rule) | TODO | 704.5j | Medium | R2 | Supertype system needed |
| R4 | Aura enchantments (attach to permanent, fall off) | BLOCKED | 303.4 | Very High | R1 | Pacifism, enchant creature, etc. |
| R5 | Activated abilities with mana costs ({2}: Pump) | TODO | 602.1 | High | - | Currently only {T} cost supported |
| R6 | Equipment (attach to creature, equip cost) | BLOCKED | 301.5 | Very High | R4, R5 | Depends on attachment + mana activation |
| R7 | Sacrifice mechanic | DONE | 701.17 | Medium | - | Action::Sacrifice moves permanent to graveyard |
| R8 | Planeswalker mechanics (loyalty, abilities, attackable) | TODO | 306 | Very High | R5 | Loyalty counters, +N/-N abilities |
| R9 | Token creation | TODO | 111 | High | - | RulesAction::CreateToken exists but not implemented |
| R10 | Triggered abilities should use the stack | TODO | 603.3 | High | - | Currently execute immediately, can't be responded to |
| R11 | Replacement effects ("instead" effects) | TODO | 614-615 | Very High | - | Fundamental to many cards |
| R12 | Prevention effects (prevent damage) | TODO | 615 | High | R11 | Often implemented together |
| R13 | Hand size enforcement at Cleanup (discard to 7) | DONE | 514.1 | Low | - | Auto-discard last cards (MVP) |
| R14 | Color identity on cards (derived from mana cost) | DONE | 105 | Low | - | colors() and is_colorless() on CardDefinition |
| R15 | Multiple activated abilities per card | TODO | 602 | Medium | R5 | Currently `Option<ActivatedAbility>` (singular) |

### Phase 7: Keywords & Abilities

Static abilities and keywords not yet implemented.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K1 | Double Strike | DONE | 702.4 | Medium | - | Implemented in C2 |
| K2 | Menace (must be blocked by 2+) | BLOCKED | 702.110 | Medium | C6 | Needs multiple blockers |
| K3 | Indestructible | DONE | 702.12 | Medium | - | Survives lethal damage + deathtouch, still dies to 0 toughness |
| K4 | Hexproof | DONE | 702.11 | Medium | - | Opponents can't target; controller can |
| K5 | Shroud | DONE | 702.18 | Medium | K4 | Can't be targeted by anyone, including controller |
| K6 | Protection from X | TODO | 702.16 | Very High | C6, R1 | DEBT: Damage, Enchanting, Blocking, Targeting |
| K7 | "Does not untap" ability | DONE | 302.6 | Low | - | StaticAbility::DoesNotUntap skips auto-untap |
| K8 | "Enters tapped" ability | DONE | - | Low | - | StaticAbility::EntersTapped, applied at enter_battlefield |
| K9 | "Cannot attack/block" effects | TODO | 508.1d | Medium | - | Pacifism-style effects |
| K10 | "Must attack" effects | TODO | 508.1d | Medium | - | Forced attack |

### Phase 8: Mana System Expansion

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| MA1 | X costs in mana (Fireball, etc.) | TODO | 107.3 | High | - | Player chooses X at cast time |
| MA2 | Hybrid mana costs ({W/U}) | TODO | 107.4e | Medium | - | |
| MA3 | Phyrexian mana (pay life instead) | TODO | 107.4f | Medium | - | |
| MA4 | Convoke (tap creatures to pay) | TODO | 702.50 | High | - | |
| MA5 | Snow mana | TODO | 107.4h | Low | - | Niche |

### Phase 9: Minor Mechanics

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| MM1 | Scry mechanic | TODO | 701.18 | Low | - | |
| MM2 | Mill mechanic | TODO | 701.13 | Low | - | |
| MM3 | Discard mechanic (forced discard) | TODO | - | Low | - | |
| MM4 | Control change (Act of Treason) | TODO | 701.10 | High | - | Owner ≠ controller |
| MM5 | "Enters tapped" for lands | DONE | - | Low | K8 | Covered by K8 (EntersTapped static ability) |

### Bugs

| # | Description | Status | Dependency | Notes |
|---|-------------|--------|------------|-------|
| B1 | Error "game is not in mulligan phase" after Keep | DONE | - | Stale mulligan messages filtered |

---

## Completed Work

### Rust Migration (March 2026)

Full migration from TypeScript/Next.js to Rust/Bevy.

### Core Engine

- Turns and phases (13 steps, 5 phases)
- Priority and stack (LIFO resolution, priority passing)
- Zones: Hand, Battlefield, Graveyard, Library, Exile, Stack
- Game setup (deck loading, shuffle, draw 7)
- Win/lose conditions (life total, empty library, draws)
- DDD Architecture: command handlers, specifications, services, value objects
- ETB triggers (detect + execute via CLIPS)
- Combat: declare attackers/blockers, damage, cleanup
- Mana pool + mana abilities (tap land/artifact → add mana)
- Spell timing validation (instant vs sorcery) + Flash keyword
- Mana cost payment with auto-pay algorithm
- Static keywords: Flying, Reach, Vigilance, Haste, First Strike, Trample, Deathtouch, Lifelink
- Summoning sickness
- +1/+1 counters and P/T calculation
- Continuous effects (temporary P/T mods, expire at Cleanup)
- Vancouver mulligan with put-back

### Bevy UI

- Battlefield display with horizontal scroll (player + opponent zones)
- Hand display (name, type, P/T, colored borders)
- Play land interaction (click → moves to battlefield)
- Cast creature/spell interaction (click → pay mana → resolves)
- Tap land/artifact for mana (click → taps, mana added)
- Combat UI (declare attackers with red border, blockers with blue)
- HUD panel (turn/step, priority, life totals, mana pool, zone counts)
- Pass Priority / End Turn buttons
- Bot-driven P2 (greedy AI: lands, mana, cast, attack)
- Fixed P1 perspective (no more hotseat confusion)
- HUD shows whose turn it is ("Player 2's Turn")
- Instant-speed casting during opponent's turn
- Target selection (creature/player targeting for damage spells)
- Card detail view on hover (name, type, cost, oracle text, P/T)
- Stack display (centered overlay when non-empty)
- Graveyard viewer (clickable toggle)
- Exile zone viewer (clickable toggle)
- Mulligan screen (Keep/Mulligan buttons, put-back card selection)
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
- Replaced hotseat with bot + fixed perspective

### CLIPS Integration (March 2026)

- clips-sys crate: CLIPS 6.4.2 compiled from C source
- SBA rules in CLIPS: correct results, ~7-11µs per cycle
- Architecture design document complete
- Full spell resolution pipeline: cast → stack → CLIPS → effects
- Cards: Lightning Strike, Giant Growth, Divination, Sol Ring, Wild Bounty

---

## Known Limitations

| Limitation | Reason | Resolved by |
|------------|--------|-------------|
| Mana pools don't empty per step | Implementation gap | C1 |
| No full 7-layer system (only temporary P/T mods) | P4 covers basics | Future (lords, static enchantments) |
| 1 blocker per attacker | MVP simplification | C6 |
| Bot doesn't block | MVP simplification | Future |
| Bot targets opponent only | No creature targeting in bot | Future |
| No counterspells | Target system too limited | C3 (blocked by R1) |
| Triggered abilities skip the stack | Execute immediately via CLIPS | R10 |
| No target legality check at resolution | Spells can't fizzle | C4 |
| SBA runs once, not loop | Implementation gap | C5 |
| No hand size enforcement | Missing from Cleanup | R13 |
| No card subtypes | Not implemented | R2 |
| Auto-pass stops for potential plays without valid targets | Conservative heuristic | Future (auto-yield) |

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
