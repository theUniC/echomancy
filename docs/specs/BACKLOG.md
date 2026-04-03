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
| R2 | Card subtypes (creature types, land types, etc.) | DONE | 205.3 | Medium | - | String-based subtypes, has_subtype(), UI type line |
| R3 | Legendary rule (legend rule) | DONE | 704.5j | Medium | R2 | is_legendary flag, SBA keeps oldest, per-player |
| R4 | Aura enchantments (attach to permanent, fall off) | BLOCKED | 303.4 | Very High | R1 | Pacifism, enchant creature, etc. |
| R5 | Activated abilities with mana costs ({2}: Pump) | DONE | 602.1 | High | - | ActivationCost::Mana + TapAndMana added |
| R6 | Equipment (attach to creature, equip cost) | BLOCKED | 301.5 | Very High | R4, R5 | Depends on attachment + mana activation |
| R7 | Sacrifice mechanic | DONE | 701.17 | Medium | - | Action::Sacrifice moves permanent to graveyard |
| R8 | Planeswalker mechanics (loyalty, abilities, attackable) | TODO | 306 | Very High | - | Loyalty counters, +N/-N abilities |
| R9 | Token creation | DONE | 111 | High | - | Game::create_token() + RulesAction::CreateToken wired |
| R10 | Triggered abilities should use the stack | DONE | 603.3 | High | - | AbilityKind::Triggered + resolve_ability calls CLIPS at resolution time |
| R11 | Replacement effects ("instead" effects) | TODO | 614-615 | Very High | - | Fundamental to many cards |
| R12 | Prevention effects (prevent damage) | TODO | 615 | High | R11 | Often implemented together |
| R13 | Hand size enforcement at Cleanup (discard to 7) | DONE | 514.1 | Low | - | Auto-discard last cards (MVP) |
| R14 | Color identity on cards (derived from mana cost) | DONE | 105 | Low | - | colors() and is_colorless() on CardDefinition |
| R15 | Multiple activated abilities per card | DONE | 602 | Medium | R5 | `Vec<ActivatedAbility>` with `ability_index` in ActivateAbility action |

### Phase 7: Keywords & Abilities

Static abilities and keywords not yet implemented.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K1 | Double Strike | DONE | 702.4 | Medium | - | Implemented in C2 |
| K2 | Menace (must be blocked by 2+) | DONE | 702.110 | Medium | C6 | Attacker with <2 blockers treated as unblocked |
| K3 | Indestructible | DONE | 702.12 | Medium | - | Survives lethal damage + deathtouch, still dies to 0 toughness |
| K4 | Hexproof | DONE | 702.11 | Medium | - | Opponents can't target; controller can |
| K5 | Shroud | DONE | 702.18 | Medium | K4 | Can't be targeted by anyone, including controller |
| K6 | Protection from X | TODO | 702.16 | Very High | C6, R1 | DEBT: Damage, Enchanting, Blocking, Targeting |
| K7 | "Does not untap" ability | DONE | 302.6 | Low | - | StaticAbility::DoesNotUntap skips auto-untap |
| K8 | "Enters tapped" ability | DONE | - | Low | - | StaticAbility::EntersTapped, applied at enter_battlefield |
| K9 | "Cannot attack/block" effects | DONE | 508.1d | Medium | - | CannotAttack + CannotBlock static abilities |
| K10 | "Must attack" effects | DONE | 508.1d | Medium | - | MustAttack flag; bot already attacks all; CannotAttack/Block filtered in legal_actions |

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
| MM1 | Scry mechanic | DONE | 701.18 | Low | - | scry/scry_with_choices + RulesAction::Scry |
| MM2 | Mill mechanic | DONE | 701.13 | Low | - | Game::mill + RulesAction::Mill |
| MM3 | Discard mechanic (forced discard) | DONE | 701.8 | Low | - | discard (specific) + discard_random + RulesAction::Discard |
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
| Triggered abilities skip the stack | Fixed — now use the stack (CR 603.3) | R10 DONE |
| No target legality check at resolution | Spells can't fizzle | C4 |
| SBA runs once, not loop | Implementation gap | C5 |
| No hand size enforcement | Missing from Cleanup | R13 |
| No card subtypes | Not implemented | R2 |
| Auto-pass stops for potential plays without valid targets | Conservative heuristic | Future (auto-yield) |
| No Defender keyword (CannotAttack used instead) | Informal equivalent exists | K11.1 |
| No Ward keyword | Not implemented | K11.2 |
| No poison counters / 10-poison loss rule | Snapshot field exists but no gameplay logic | P10.1 |
| No replacement effects (CR 614) | R11 TODO | R11 |
| No prevention effects (CR 615) | R12 TODO | R12 |
| Continuous effects only cover temporary P/T mods | No layers 1-6, no static auras/lords | P10.4 |
| No copy effects (CR 707) | Not implemented | P10.5 |
| No extra turns or extra phases (CR 500.7, 723) | Not implemented | P10.7 |
| No modal spells (choose one/two) | Not implemented | P10.8 |
| No bounce (return to hand) RulesAction | Not implemented | P10.10 |
| No fight mechanic (CR 701.14) | Not implemented | P10.12 |
| No mana abilities vs non-mana abilities distinction | Partial (Effect::is_mana_ability) | P10.3 |
| Trigger system limited to 4 event types | ZoneChanged, StepStarted, AttackerDeclared, CombatEnded only | P10.9 |
| No -1/-1 counters or counter annihilation SBA (CR 704.5q) | Not implemented | P10.6 |

---

### Phase 10: Missing Core Game Systems

Fundamental MTG game systems not yet tracked in any phase. Ordered by dependency.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P10.1 | Poison counters and 10-poison loss rule | TODO | 704.5c, 122.1b | Medium | - | SBA check: player with 10+ poison loses. Snapshot field already exists but no gameplay logic |
| P10.2 | -1/-1 counters | TODO | 122.1b | Medium | - | Counter type on creatures, affects P/T calculation |
| P10.3 | Mana abilities vs non-mana abilities formal distinction | TODO | 605 | Medium | - | Partial: Effect::is_mana_ability exists. Need: triggered mana abilities, mana ability timing rules during casting (CR 605.3a) |
| P10.4 | Full 7-layer continuous effects system | TODO | 613 | Very High | - | Currently only temporary P/T mods (layer 7c). Need all 7 layers + sublayers, timestamps, dependency system |
| P10.5 | Copy effects | TODO | 707 | Very High | P10.4 | Clone, Copy Enchantment, etc. Applies in layer 1. Requires copiable values concept |
| P10.6 | +1/+1 and -1/-1 counter annihilation SBA | TODO | 704.5q | Low | P10.2 | When both exist on a permanent, remove pairs. Already have +1/+1 counters |
| P10.7 | Extra turns and extra phases | TODO | 500.7, 723 | High | - | Time Walk, extra combat phases. Turn queue system |
| P10.8 | Modal spells (choose one / choose two) | TODO | 700.2 | High | - | Requires mode selection at cast time (CR 700.2a). Many common cards are modal |
| P10.9 | Expanded trigger event types | TODO | 603.1 | High | - | Currently only 4 types. Need: damage dealt, life gained/lost, spell cast, creature died, land played, counters placed, etc. |
| P10.10 | Bounce (return to hand) effect | TODO | 701.3 | Low | - | Unsummon-type effects. Need RulesAction::ReturnToHand |
| P10.11 | Exile-and-return (flicker) effects | TODO | - | Medium | - | Flickerwisp, Restoration Angel. Exile then return to battlefield. Needs delayed trigger system |
| P10.12 | Fight mechanic | TODO | 701.14 | Medium | - | Each creature deals damage equal to its power to the other. Common in green |
| P10.13 | Goad mechanic | TODO | 701.15 | Medium | - | Goaded creature must attack and must attack someone other than goader. Multiplayer-relevant |
| P10.14 | Proliferate | TODO | 701.34 | Medium | P10.1, P10.2 | Choose any number of permanents/players with counters, add one of each kind. Very common in recent sets |
| P10.15 | Investigate / Clue tokens | TODO | 701.16 | Low | R9 | Create predefined Clue artifact token with "{2}, Sacrifice: Draw a card" |
| P10.16 | Food tokens | TODO | - | Low | R9 | Predefined token: "{2}, {T}, Sacrifice: Gain 3 life" |
| P10.17 | Treasure tokens | TODO | - | Low | R9 | Predefined token: "{T}, Sacrifice: Add one mana of any color." Already very common |
| P10.18 | Blood tokens | TODO | - | Low | R9 | Predefined token: "{1}, {T}, Discard a card, Sacrifice: Draw a card" |
| P10.19 | Map tokens | TODO | - | Low | R9 | Predefined token from recent sets |
| P10.20 | Delayed triggers | TODO | 603.7 | High | - | "At the beginning of next end step, return it" — requires scheduling future triggers |
| P10.21 | "As ~ enters" / choice on entry effects | TODO | 614.1c | High | R11 | Replacement effects that ask for a choice when entering battlefield (e.g., Clone choosing what to copy) |
| P10.22 | Owner vs controller distinction for all zones | TODO | 108.3 | Medium | - | Currently partial. Cards must return to owner's graveyard/hand, not controller's |
| P10.23 | Mana of any color production | TODO | 106.1 | Medium | - | Many cards add "one mana of any color." Requires color choice at resolution |
| P10.24 | Colorless mana ({C}) as distinct from generic | TODO | 107.4c | Low | - | Wastes, Eldrazi cards require specifically colorless mana |
| P10.25 | State-based actions completeness | TODO | 704.5 | Medium | P10.1 | Missing: 704.5c (poison), 704.5d (token in wrong zone), 704.5e (copy in wrong zone), 704.5i (planeswalker 0 loyalty), 704.5m (illegal aura), 704.5n (illegal equipment), 704.5q (counter annihilation), 704.5s (saga sacrifice), 704.5v (battle 0 defense) |

### Phase 11: Keyword Abilities (Extended)

Keywords from CR 702 not yet tracked. Grouped by frequency in Standard/Modern play.

**Tier 1: Very Common (appear in most sets)**

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K11.1 | Defender | TODO | 702.3 | Low | - | "This creature can't attack." Functionally equivalent to CannotAttack but is the official keyword. Should be StaticAbility variant |
| K11.2 | Ward {N} | TODO | 702.21 | Medium | - | Triggered ability: when targeted by opponent, counter unless they pay cost. Very common since 2021 |
| K11.3 | Prowess | TODO | 702.108 | Medium | P10.9 | Triggered: whenever you cast a noncreature spell, this gets +1/+1 until EOT. Needs "spell cast" trigger event |
| K11.4 | Protection from X | TODO | 702.16 | Very High | C6, R1 | DEBT: Damage prevented, Enchanting/Equipping falls off, Blocking prevented, Targeting prevented. Already tracked as K6 |
| K11.5 | Fear | TODO | 702.36 | Low | - | Can't be blocked except by artifact creatures and/or black creatures. Legacy keyword |
| K11.6 | Intimidate | TODO | 702.13 | Low | R14 | Can't be blocked except by artifact creatures and/or creatures sharing a color. Requires color identity checks |
| K11.7 | Skulk | TODO | 702.118 | Low | - | Can't be blocked by creatures with greater power |
| K11.8 | Shadow | TODO | 702.28 | Low | - | Can only block/be blocked by creatures with shadow |
| K11.9 | Horsemanship | TODO | 702.31 | Low | - | Can only block/be blocked by creatures with horsemanship (flying variant for Portal sets) |

**Tier 2: Common (appear in many sets)**

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K11.10 | Undying | TODO | 702.93 | Medium | P10.2 | When dies with no +1/+1 counters, return with a +1/+1 counter. Needs -1/-1 interaction awareness |
| K11.11 | Persist | TODO | 702.79 | Medium | P10.2 | When dies with no -1/-1 counters, return with a -1/-1 counter. Needs -1/-1 counters |
| K11.12 | Wither | TODO | 702.80 | Medium | P10.2 | Damage dealt to creatures is dealt as -1/-1 counters instead |
| K11.13 | Infect | TODO | 702.90 | Medium | P10.1, P10.2 | Damage to creatures as -1/-1, damage to players as poison counters |
| K11.14 | Toxic N | TODO | 702.164 | Low | P10.1 | Whenever this deals combat damage to a player, that player gets N poison counters |
| K11.15 | Evolve | TODO | 702.100 | Medium | P10.9 | Triggered: when a creature ETBs under your control with greater P or T, put +1/+1 counter |
| K11.16 | Exploit | TODO | 702.110 | Medium | R7 | When ETBs, you may sacrifice a creature. Cards have "when ~ exploits" trigger |
| K11.17 | Fabricate N | TODO | 702.123 | Medium | R9 | ETB: choose +1/+1 counters or create N 1/1 Servo tokens |
| K11.18 | Afflict N | TODO | 702.130 | Medium | P10.9 | Triggered: whenever this becomes blocked, defending player loses N life |
| K11.19 | Mentor | TODO | 702.134 | Medium | P10.9 | Triggered: when attacks with creature of lesser power, put +1/+1 counter on it |
| K11.20 | Training | TODO | 702.149 | Medium | P10.9 | Triggered: when attacks with creature of greater power, put +1/+1 counter on this |
| K11.21 | Exalted | TODO | 702.83 | Medium | P10.9 | Triggered: when a creature you control attacks alone, it gets +1/+1 until EOT |
| K11.22 | Battle Cry | TODO | 702.91 | Medium | P10.9 | Triggered: when attacks, other attacking creatures get +1/+0 until EOT |
| K11.23 | Annihilator N | TODO | 702.86 | Medium | R7, P10.9 | Triggered: when attacks, defending player sacrifices N permanents |
| K11.24 | Dethrone | TODO | 702.105 | Medium | P10.9 | Triggered: when attacks the player with most life, put +1/+1 counter |
| K11.25 | Renown N | TODO | 702.112 | Medium | P10.9 | Triggered: when deals combat damage to player, if not renowned, put N +1/+1 counters, becomes renowned |
| K11.26 | Melee | TODO | 702.121 | Medium | P10.9 | Triggered: when attacks, +1/+1 until EOT for each opponent attacked this combat. Multiplayer |
| K11.27 | Flanking | TODO | 702.25 | Low | P10.9 | Triggered: when blocked by creature without flanking, blocker gets -1/-1 until EOT |
| K11.28 | Bushido N | TODO | 702.45 | Low | P10.9 | Triggered: when blocks or becomes blocked, gets +N/+N until EOT |
| K11.29 | Myriad | TODO | 702.116 | High | R9, P10.9 | Triggered: when attacks, create token copies attacking each other opponent. Multiplayer |
| K11.30 | Enlist | TODO | 702.154 | Medium | P10.9 | When attacks, tap non-attacking creature to add its power. Recent keyword |

**Tier 3: Evasion and Static (blocking restrictions)**

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K11.31 | Phasing | TODO | 702.26 | Very High | - | Phases out on untap step, phases back in next untap. Treated as though it doesn't exist. Complex zone-like behavior |
| K11.32 | Changeling | TODO | 702.73 | Low | R2 | Has all creature types. Simple flag on subtypes |
| K11.33 | Devoid | TODO | 702.114 | Low | R14 | Colorless regardless of mana cost. Characteristic-defining ability in layer 5 |

**Tier 4: Triggered on death / graveyard (common patterns)**

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K11.34 | Afterlife N | TODO | 702.135 | Low | R9, P10.9 | When dies, create N 1/1 white and black Spirit tokens with flying |
| K11.35 | Decayed | TODO | 702.147 | Low | - | Can't block. When attacks, sacrifice at end of combat. Common on zombie tokens |
| K11.36 | Modular N | TODO | 702.43 | Medium | P10.9 | ETBs with N +1/+1 counters. When dies, move counters to target artifact creature |
| K11.37 | Living Weapon | TODO | 702.92 | Medium | R6, R9 | ETBs and creates 0/0 Phyrexian Germ token, attaches to it. Requires Equipment |
| K11.38 | Unearth {cost} | TODO | 702.84 | Medium | - | Activated from graveyard: return to battlefield with haste, exile at EOT or if it would leave |
| K11.39 | Encore {cost} | TODO | 702.141 | High | R9 | Activated from graveyard: create token copies for each opponent, exile card |
| K11.40 | Escape {cost} (exile N cards) | TODO | 702.138 | Medium | - | Cast from graveyard for escape cost + exile cards from graveyard |
| K11.41 | Disturb {cost} | TODO | 702.146 | Medium | - | Cast transformed from graveyard. Requires DFC support |

### Phase 12: Advanced Card Types

Complex card layouts and types not yet tracked.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P12.1 | Planeswalker mechanics (loyalty, abilities, attackable) | TODO | 306 | Very High | - | Already tracked as R8. Loyalty counters, +N/-N abilities, one activation per turn, can be attacked. SBA: 0 loyalty = graveyard |
| P12.2 | Double-faced cards (transform/convert) | TODO | 712 | Very High | - | Front/back faces, transform keyword action (CR 701.27). Daybound/Nightbound variant |
| P12.3 | Modal double-faced cards (MDFCs) | TODO | 712.3 | Very High | P12.2 | Choose which face to play. Zendikar Rising lands, Strixhaven deans. Each face independent |
| P12.4 | Split cards | TODO | 709 | High | - | Two halves on one card. Fuse variant (CR 702.102) allows casting both halves |
| P12.5 | Saga enchantments | TODO | 714 | High | P10.9 | Lore counters, chapter abilities trigger on lore counter placement. SBA: sacrifice when chapters exhausted (CR 704.5s) |
| P12.6 | Adventure cards | TODO | 715 | High | - | Cast as Adventure (instant/sorcery) from hand, exiled on resolution, then cast creature from exile |
| P12.7 | Class enchantments | TODO | 716 | High | R5 | Level-up activated abilities that unlock static abilities. Three levels |
| P12.8 | Vehicles (artifact subtype) + Crew | TODO | 702.122, 301.7 | High | R2 | Artifact that becomes creature when crewed. Crew N: tap creatures with total power N+ |
| P12.9 | Battles (card type) | TODO | 310 | Very High | - | New card type (March of the Machine). Defense counters, protector, can be attacked. SBA: 0 defense = graveyard |
| P12.10 | Dungeons | TODO | 309 | High | - | Command zone cards with rooms. Venture mechanic moves through rooms triggering abilities |
| P12.11 | Meld cards | TODO | 712.4, 701.42 | Very High | P12.2 | Two specific cards combine into one oversized card. Seven known meld pairs |
| P12.12 | Leveler cards (Level Up) | TODO | 711, 702.87 | Medium | R5 | Level counters unlock abilities at thresholds |
| P12.13 | Face-down spells and permanents | TODO | 708 | Very High | - | Morph, Megamorph, Manifest, Cloak, Disguise. 2/2 face-down creature with no abilities. Complex reveal timing |
| P12.14 | Prototype cards | TODO | 718, 702.160 | Medium | P10.4 | Can be cast for alternative characteristics (different cost, P/T, color). Layer 1 interaction |
| P12.15 | Kindred card type | TODO | 308 | Low | R2 | "Kindred" type that has creature subtypes (e.g., Kindred Sorcery - Goblin). Renamed from "Tribal" |
| P12.16 | Day/Night and Daybound/Nightbound | TODO | 730, 702.145 | High | P12.2 | Global game state tracking day/night. Creatures transform based on it |

### Phase 13: Alternative Costs and Casting

Mechanics that modify how spells are cast (alternative costs, additional costs, cost reduction).

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P13.1 | Kicker / Multikicker | TODO | 702.33 | High | - | Optional additional cost. If kicked, spell has enhanced effect. Very common across sets |
| P13.2 | Flashback {cost} | TODO | 702.34 | Medium | - | Cast from graveyard for flashback cost, exile afterward. Extremely common |
| P13.3 | Cycling {cost} | TODO | 702.29 | Medium | - | Activated from hand: pay cost, discard this, draw a card. Very common. Has "when you cycle" triggers |
| P13.4 | Overload {cost} | TODO | 702.96 | Medium | - | Alternative cost: replace "target" with "each." Changes single-target to board-wide |
| P13.5 | Bestow {cost} | TODO | 702.103 | Very High | R4 | Cast as Aura for bestow cost; if enchanted creature leaves, becomes creature. Complex type-changing |
| P13.6 | Evoke {cost} | TODO | 702.74 | Medium | R7 | Alternative cost, sacrifice when ETBs. Common in Elementals (MH2 cycle) |
| P13.7 | Madness {cost} | TODO | 702.35 | High | - | When discarded, may cast for madness cost instead of going to graveyard. Needs replacement effect on discard |
| P13.8 | Emerge {cost} (sacrifice + reduce) | TODO | 702.119 | High | R7 | Alternative cost: sacrifice creature, reduce emerge cost by sacrificed creature's mana value |
| P13.9 | Delve | TODO | 702.66 | Medium | - | Exile cards from graveyard to pay {1} each toward generic mana cost. Very powerful |
| P13.10 | Convoke | TODO | 702.51 | High | - | Already tracked as MA4. Tap creatures to pay mana. Each tapped creature pays {1} or one mana of its color |
| P13.11 | Improvise | TODO | 702.126 | Medium | - | Tap artifacts to pay {1} each toward generic cost. Artifact version of Convoke |
| P13.12 | Affinity for [type] | TODO | 702.41 | Medium | - | Costs {1} less for each [type] you control. Most commonly "Affinity for artifacts" |
| P13.13 | Cascade | TODO | 702.85 | Very High | - | When cast, exile from top until nonland with lesser mana value found, cast it free. Recursive potential |
| P13.14 | Storm | TODO | 702.40 | High | P10.9 | When cast, copy for each spell cast before it this turn. Needs storm count tracking |
| P13.15 | Buyback {cost} | TODO | 702.27 | Medium | - | Additional cost: if paid, return to hand instead of graveyard on resolution |
| P13.16 | Retrace | TODO | 702.81 | Medium | - | Cast from graveyard by discarding a land as additional cost |
| P13.17 | Spectacle {cost} | TODO | 702.137 | Low | P10.9 | Alternative cost if opponent lost life this turn. Needs "life lost" tracking |
| P13.18 | Foretell {cost} | TODO | 702.143 | Medium | - | Pay {2} to exile face-down from hand. Later cast for foretell cost from exile |
| P13.19 | Dash {cost} | TODO | 702.109 | Medium | - | Alternative cost: creature has haste, return to hand at end of turn |
| P13.20 | Blitz {cost} | TODO | 702.152 | Medium | P10.20 | Alternative cost: creature has haste, draw when dies, sacrifice at EOT |
| P13.21 | Mutate {cost} | TODO | 702.140 | Very High | - | Cast on top/bottom of non-Human creature. Merge into one creature with combined abilities. Very complex |
| P13.22 | Ninjutsu {cost} | TODO | 702.49 | High | - | Activated from hand: return unblocked attacking creature, put this onto battlefield attacking. Timing-specific |
| P13.23 | Splice onto [type] {cost} | TODO | 702.47 | High | - | Reveal from hand, add text to spell of matching type. Card stays in hand. Complex rules |
| P13.24 | Prowl {cost} | TODO | 702.76 | Low | R2 | Alternative cost if creature of shared type dealt combat damage this turn |
| P13.25 | Spree | TODO | 702.172 | Medium | P10.8 | Cast with spree, choose modes and pay additional costs per mode. Recent (Outlaws of Thunder Junction) |
| P13.26 | Bargain | TODO | 702.166 | Medium | R7 | Optional additional cost: sacrifice an artifact, enchantment, or token. Recent keyword |
| P13.27 | Casualty N | TODO | 702.153 | Medium | R7 | Additional cost: sacrifice creature with power N+. If paid, copy the spell |
| P13.28 | Craft {cost} (exile materials) | TODO | 702.167 | Medium | P12.2 | Activated from battlefield: exile this + materials, return transformed. Requires DFC |
| P13.29 | Plot {cost} | TODO | 702.170 | Medium | - | Pay plot cost to exile from hand face-up. Cast from exile on later turn without paying mana cost |
| P13.30 | Offspring {cost} | TODO | 702.175 | Medium | R9 | Additional cost: if paid, create 1/1 token copy when ETBs. Recent keyword |
| P13.31 | Suspend N — {cost} | TODO | 702.62 | High | P10.20 | Exile from hand with N time counters. Remove one each upkeep. Cast when last removed. Needs delayed trigger system |
| P13.32 | Alternative casting from zones (general) | TODO | 601.3 | High | - | Framework for casting from graveyard, exile, library top. Needed by Flashback, Escape, Foretell, etc. |

### Phase 14: Multiplayer and Format Rules

Rules for multiplayer games and specific formats.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P14.1 | 3+ player support (multiplayer) | TODO | 800-810 | Very High | - | APNAP order, range of influence, attack directions. Foundation for Commander |
| P14.2 | Commander format rules | TODO | 903 | Very High | P14.1, R14 | 100-card singleton, commander zone, color identity, commander tax, 21 commander damage |
| P14.3 | The Monarch mechanic | TODO | 724 | Medium | P14.1 | Designated player draws extra card at end step. Deals combat damage to monarch = become monarch |
| P14.4 | The Initiative mechanic | TODO | 725 | High | P14.1, P12.10 | Similar to Monarch, but enters Undercity dungeon. Recent mechanic |
| P14.5 | Companion (deckbuilding restriction) | TODO | 702.139 | High | - | Declared before game. Can be cast once from sideboard if deck meets restriction |
| P14.6 | The Ring Tempts You | TODO | 701.54 | High | - | Lord of the Rings mechanic. Designate ring-bearer, get cumulative abilities |
| P14.7 | Two-Headed Giant format | TODO | 810 | High | P14.1 | Shared turns, shared life total (30), modified SBAs |

### Phase 15: Keyword Actions (Extended)

Keyword actions from CR 701 not yet implemented (beyond scry, mill, discard, sacrifice).

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P15.1 | Surveil N | TODO | 701.25 | Low | - | Like scry but cards go to graveyard instead of bottom. Very common in recent sets |
| P15.2 | Explore | TODO | 701.44 | Medium | - | Reveal top card; if land put in hand, otherwise +1/+1 counter and choose to keep/put to GY. Common |
| P15.3 | Connive N | TODO | 701.50 | Medium | - | Draw N, discard N, get +1/+1 counter for each nonland discarded. Streets of New Capenna |
| P15.4 | Populate | TODO | 701.36 | Medium | R9 | Create a copy of a creature token you control. Selesnya mechanic |
| P15.5 | Amass [type] N | TODO | 701.47 | Medium | R9, R2 | Put N +1/+1 counters on army you control, or create 0/0 Army token first |
| P15.6 | Bolster N | TODO | 701.39 | Low | - | Put N +1/+1 counters on creature you control with least toughness |
| P15.7 | Adapt N | TODO | 701.46 | Low | - | If no +1/+1 counters, put N +1/+1 counters. RNA mechanic |
| P15.8 | Manifest / Manifest Dread | TODO | 701.40, 701.62 | High | P12.13 | Put card from top of library face-down as 2/2. Turn face-up for cost. Needs face-down system |
| P15.9 | Cloak | TODO | 701.58 | High | P12.13 | Like manifest but card gets ward {2} while face-down |
| P15.10 | Incubate N | TODO | 701.53 | Medium | R9 | Create Incubator artifact token with N +1/+1 counters. Pay {2}: transform into Phyrexian creature |
| P15.11 | Exert | TODO | 701.43 | Low | - | Choose to exert when attacking; doesn't untap next untap step. Common in Amonkhet |
| P15.12 | Suspect | TODO | 701.60 | Low | - | Gains menace and can't block. Recent Streets of New Capenna: Murders mechanic |
| P15.13 | Detain | TODO | 701.35 | Low | - | Can't attack, block, or activate abilities until your next turn |
| P15.14 | Transform keyword action | TODO | 701.27 | High | P12.2 | Turn a DFC over. Needed by many Phase 12 items |
| P15.15 | Forage | TODO | 701.61 | Low | R7 | "Sacrifice a Food or exile three cards from graveyard." Recent Bloomburrow keyword action |
| P15.16 | Collect Evidence N | TODO | 701.59 | Low | - | Exile cards from graveyard with total mana value N+. Murders at Karlov Manor mechanic |
| P15.17 | Discover N | TODO | 701.57 | High | P13.13 | Like cascade but with a mana value limit. Cast or put into hand. Recent keyword action |

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
