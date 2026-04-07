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
| C7 | Concede (player can concede at any time → loses immediately) | DONE | 104.3a | Low | - | Action::Concede + GameEndReason::Concession + UI button |

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
| R11 | Replacement effects ("instead" effects) | DONE | 614-615 | Very High | - | MVP: damage prevention, destroy/regen, ETB. Framework + registry + interception |
| R12 | Prevention effects (prevent damage) | DONE | 615 | High | R11 ✅ | Fog (all combat), Guardian Shield (all to target), is_combat flag, AllCombatDamage filter |
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
| MA1 | X costs in mana (Fireball, etc.) | DONE | 107.3 | High | - | Player chooses X at cast time; x field on ManaCost, x_value on Action::CastSpell and SpellOnStack |
| MA2 | Hybrid mana costs ({W/U}) | TODO | 107.4e | Medium | - | |
| MA3 | Phyrexian mana (pay life instead) | TODO | 107.4f | Medium | - | |
| MA4 | Convoke (tap creatures to pay) | TODO | 702.50 | High | - | |
| MA5 | Snow mana | DONE | 107.4h | Low | - | is_snow flag on CardDefinition + with_snow() builder; no snow mana payment logic yet |
| MA6 | Smart mana auto-pay (preserve colored mana for upcoming spells) | TODO | - | Medium | - | Current auto-pay spends colored mana on generic costs; should prefer colorless/generic first to preserve colored for later casts |

### Phase 9: Minor Mechanics

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| MM1 | Scry mechanic | DONE | 701.18 | Low | - | scry/scry_with_choices + RulesAction::Scry |
| MM2 | Mill mechanic | DONE | 701.13 | Low | - | Game::mill + RulesAction::Mill |
| MM3 | Discard mechanic (forced discard) | DONE | 701.8 | Low | - | discard (specific) + discard_random + RulesAction::Discard |
| MM4 | Control change (Act of Treason) | TODO | 701.10 | High | - | Owner ≠ controller |
| MM5 | "Enters tapped" for lands | DONE | - | Low | K8 | Covered by K8 (EntersTapped static ability) |

### Phase 10: Core Rules Systems

Fundamental rules infrastructure missing from the engine. Most card interactions require these.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| LS1 | Layer system (continuous effects ordering) | DONE | 613 | Very High | - | Layers 4-7. Pure-function pipeline, all ability/P/T reads migrated |
| CP1 | Copy effects (copy spell / copy permanent) | TODO | 706 | Very High | LS1 | Fork, Twincast, Clone, etc. Copies use the layer system |
| MD1 | Modal spells (choose one / choose two / choose all) | TODO | 700.2 | Medium | - | Very common. Needs choice UI and cost system hook |
| SC1 | Split cards (two spells on one card) | TODO | 708 | Medium | MD1 | Fuse support optional |
| SA1 | Sagas (lore counters, chapter triggers) | TODO | 715 | High | - | New card type. Add counter on ETB + upkeep, trigger per chapter |
| VH1 | Vehicles (Artifact — Vehicle, Crew N) | TODO | 301.7 | High | - | Tap creatures with total power ≥ N → becomes creature until EOT |

### Phase 11: Triggered Ability Patterns

The CLIPS engine can write rules for these, but the domain needs to emit the right events first.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| TR1 | Upkeep triggers ("at the beginning of your upkeep") | DONE | 502.1 | Medium | - | StepStarted event + AtStepStart condition (R10 infrastructure) |
| TR2 | End step triggers ("at the beginning of your end step") | DONE | 513.1 | Medium | - | StepStarted event + AtStepStart condition (R10 infrastructure) |
| TR3 | Death triggers ("when/whenever X dies") | DONE | 603.6 | Medium | - | ZoneChanged event + SourceDies condition (R10 infrastructure) |
| TR4 | Attack triggers ("whenever X attacks / whenever a creature attacks") | DONE | 603.6 | Low | - | CreatureDeclaredAttacker event (R10 infrastructure) |
| TR5 | Draw triggers ("whenever you draw a card") | DONE | 603.6 | Low | - | CardDrawn event emitted per draw + TriggerEventType::CardDrawn |
| TR6 | Cast triggers ("whenever you cast a spell") | DONE | 603.6 | Low | - | SpellCast event emitted at cast time + TriggerEventType::SpellCast |
| TR7 | LTB triggers (leaves-the-battlefield) | TODO | 603.6c | Medium | TR3 | Broader than death: exile, bounce, sacrifice |

### Phase 12: Alternate Casting Costs

A framework for paying alternative costs at cast time. Each keyword below is an instance.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| AK1 | Cycling ({cost}: discard → draw) | TODO | 702.28 | Low | MM3 | Special action, doesn't use the stack |
| AK2 | Flashback (cast from graveyard) | TODO | 702.33 | Medium | - | Alternate cost; exile after resolution |
| AK3 | Kicker / Multikicker (optional additional cost) | TODO | 702.32 | Medium | - | Adds {kicker_paid} flag to SpellOnStack |
| AK4 | Jump-start (flashback but discard a card) | TODO | 702.132 | Low | AK2 | Variant of Flashback |
| AK5 | Escape (cast from GY, exile N cards) | TODO | 702.156 | Medium | AK2 | Variant of Flashback |
| AK6 | Foretell (exile face-down for {2}, cast later) | TODO | 702.173 | High | - | New zone state + alternate cost next turn |
| AK7 | Overload (pay more → target all instead of one) | TODO | 702.96 | Medium | MD1 | Changes targeting mode at cast time |
| AK8 | Entwine (pay both modes of a modal spell) | TODO | 702.47 | Low | MD1 | Additive cost on modal spells |
| AK9 | Suspend (exile with time counters, cast when last removed) | TODO | 702.61 | High | TR1 | Needs upkeep trigger + exile zone state |
| AK10 | Delve (exile cards from GY to reduce cost) | TODO | 702.65 | Medium | - | Alternative cost reduction |
| AK11 | Affinity (cost reduction per permanent type) | TODO | 702.40 | Medium | - | Alternative cost reduction |
| AK12 | Emerge (sacrifice creature, reduce cost by its CMC) | TODO | 702.115 | Medium | R7 ✅ | Alternative cost + sacrifice |
| AK13 | Madness (cast when discarded for alternate cost) | TODO | 702.34 | High | MM3, AK2 | Needs discard hook + exile-cast window |
| AK14 | Cascade (cast random cheaper spell when casting) | TODO | 702.84 | High | - | Trigger on cast, exile until found |
| AK15 | Storm (copy once per spell cast this turn) | TODO | 702.39 | High | TR6 | Needs spell count per turn |
| AK16 | Buyback (pay extra → return to hand instead of GY) | TODO | 702.27 | Low | - | Alternate cost, destination change |
| AK17 | Retrace (cast from GY, discard a land) | TODO | 702.87 | Low | AK2, MM3 | Variant of Flashback |

### Phase 13: More Creature Keywords

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| CK1 | Regenerate ({cost}: next time destroyed → tap + remove damage) | DONE | 701.15 | Medium | R11 | Replacement effect; old keyword still on many cards |
| CK2 | Undying (dies with no +1/+1 counter → return with one) | TODO | 702.92 | Medium | TR3 | Death trigger + replacement |
| CK3 | Persist (dies with no -1/-1 counter → return with one) | TODO | 702.77 | Medium | TR3 | Like Undying but -1/-1 |
| CK4 | Wither (deals damage as -1/-1 counters) | TODO | 702.79 | Low | - | Damage-dealing replacement |
| CK5 | Infect (damage as -1/-1 to creatures, poison to players) | TODO | 702.89 | Medium | CK4 | Extends Toxic/Wither model; P10.1 ✅ for poison |
| CK6 | Prowess (+1/+1 until EOT whenever you cast non-creature) | TODO | 702.107 | Low | TR6 | Triggered ability off SpellCast event |
| CK7 | Exalted (attacking alone gets +1/+1) | TODO | 702.90 | Low | TR4 | Triggered ability off AttackerDeclared |
| CK8 | Landwalk (unblockable if opponent controls that land type) | TODO | 702.14 | Low | R2 ✅ | Forestwalk, Islandwalk, etc.; checks opponent lands |
| CK9 | Intimidate (blocked only by artifacts + same-color) | TODO | 702.13 | Low | - | Like Fear but color-based |
| CK10 | Annihilator N (attacker — defending player sacrifices N) | TODO | 702.85 | Medium | R7 ✅ | Triggered on attack |
| CK11 | Modular N (ETB with N +1/+1; moves counters on death) | TODO | 702.43 | Medium | TR3 | Death trigger + counter transfer |
| CK12 | Graft N (ETB with N +1/+1; can move to entering creature) | TODO | 702.57 | Medium | - | Triggered on other creature ETB |
| CK13 | Ninjutsu ({cost}: swap with unblocked attacker from hand) | TODO | 702.48 | High | - | Activated ability outside normal timing |
| CK14 | Morph / Megamorph (cast face-down for {3}) | TODO | 702.36 | Very High | LS1 | Face-down permanent state; turn face-up as special action |
| CK15 | Mutate (cast on non-human creature, merge) | TODO | 702.157 | Very High | LS1 | New merge permanent; complex layer interactions |
| CK16 | Level up ({cost}: put level counter, gains abilities) | TODO | 702.87b | High | - | Activated ability + level counter thresholds |
| CK17 | Evoke (pay evoke cost → sacrifice when ETB trigger resolves) | TODO | 702.73 | Medium | - | Alternate cost + mandatory sacrifice |
| CK18 | Champion (exile creature of type; return if leaves) | TODO | 702.71 | Medium | TR7 | ETB exile + LTB return |

### Phase 14: Ability Words & Condition Patterns

These are not keywords but common trigger conditions. CLIPS rules use them, but the domain must emit the right events (see Phase 11).

| # | Description | Status | Notes |
|---|-------------|--------|-------|
| AW1 | Landfall (whenever a land enters under your control) | TODO | Needs ETB event filtered by land type |
| AW2 | Threshold (7+ cards in your graveyard) | TODO | Condition check in CLIPS rules |
| AW3 | Hellbent (no cards in hand) | TODO | Condition check in CLIPS rules |
| AW4 | Morbid (a creature died this turn) | TODO | Needs death event flag per turn |
| AW5 | Revolt (a permanent left your control this turn) | TODO | Needs LTB event flag per turn |
| AW6 | Magecraft (whenever you cast or copy an instant/sorcery) | TODO | Needs SpellCast + Copy events |
| AW7 | Raid (you attacked this turn) | TODO | Needs attacked flag per turn |
| AW8 | Spectacle (opponent lost life this turn) | TODO | Needs opponent-life-lost flag per turn |
| AW9 | Delirium (4+ card types in GY) | TODO | Condition check in CLIPS rules |
| AW10 | Undergrowth (count creatures in GY) | TODO | Condition check in CLIPS rules |

### Phase 15: Classic Keywords Missed

Established keywords from older sets not yet in the backlog.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| CL1 | Proliferate (add counter to each permanent with a counter) | TODO | 701.27 | Low | - | Very common in counter-heavy sets |
| CL2 | Amass N (create/grow Army token with N +1/+1 counters) | TODO | 701.44 | Medium | R9 ✅ | Token of specific subtype + counter add |
| CL3 | Explore (reveal top; if land put in hand, else +1/+1 on this) | TODO | 701.39 | Medium | - | Common in Ixalan block |
| CL4 | Mentor (when attacks with bigger creature, +1/+1 on smaller attacker) | TODO | 702.134 | Low | TR4 | Triggered on attack |
| CL5 | Riot (ETB: choose haste or +1/+1 counter) | TODO | 702.138 | Low | - | ETB choice; common in Ravnica |
| CL6 | Afterlife N (when dies, create N 1/1 Spirit tokens) | TODO | 702.150 | Low | TR3, R9 ✅ | Death trigger + token creation |
| CL7 | Training (when attacks alongside creature with greater power, +1/+1) | TODO | 702.178 | Low | TR4 | Attack trigger |
| CL8 | Disturb (cast enchantment creature from graveyard, transformed) | TODO | 702.179 | High | AK2 | Variant of Flashback for DFCs |
| CL9 | Decayed (can attack, sacrificed at EOT, can't block) | TODO | 702.181 | Low | - | Static ability combo |
| CL10 | Cleave (pay extra cost to remove text in [brackets]) | TODO | 702.183 | High | - | Modifies card text at cast time |
| CL11 | Meld (two specific cards combine into one oversized permanent) | TODO | 712 | Very High | - | Special two-card pairing |
| CL12 | Embalm (exile from GY to create token copy) | TODO | 702.128 | Medium | R9 ✅ | Activated ability from GY |
| CL13 | Eternalize (like Embalm but 4/4 black token) | TODO | 702.129 | Low | CL12 | Variant of Embalm |
| CL14 | Aftermath (sorcery half castable only from GY) | TODO | 702.130 | Medium | AK2 | Split card variant |
| CL15 | Surge (alternate cost if you/teammate cast a spell this turn) | TODO | 702.118 | Medium | TR6 | SpellCast flag check |
| CL16 | Awaken (pay extra → put N +1/+1 on target land, it becomes creature) | TODO | 702.112 | Medium | - | Kicker-like extra effect on lands |
| CL17 | Renown N (first time deals combat damage to player → become renowned with N counters) | TODO | 702.111 | Low | - | Combat damage trigger |
| CL18 | Dash (cast for dash cost, haste, return to hand at EOT) | TODO | 702.109 | Medium | - | Alternate cost + ETB + end trigger |
| CL19 | Ascend / City's Blessing (if you control 10+ permanents, get the city's blessing) | TODO | 702.131 | Medium | - | Permanent flag gained via SBA-like check |
| CL20 | Converge (effect scales with number of colors of mana spent) | TODO | 702.117 | Medium | - | Mana-payment tracking at cast time |
| CL21 | Addendum (bonus if cast during your main phase) | TODO | 702.141 | Low | - | Timing condition check at resolution |

### Phase 16: Recent Mechanics (2022–2025)

Mechanics introduced in recent sets. Some require new card types or structural changes.

| # | Description | Status | Set | Complexity | Dependency | Notes |
|---|-------------|--------|-----|------------|------------|-------|
| RE1 | Craft (exile cards from hand/GY + pay cost → transform) | TODO | LCI 2023 | High | - | Activated ability on DFCs; exile specific cards as cost |
| RE2 | Plot (exile for plot cost; cast for free on later turn) | TODO | OTJ 2024 | Medium | - | Special action; exiled card gains "cast for free" flag |
| RE3 | Impending N (enters as non-creature with N time counters; becomes creature when last removed) | TODO | BLB 2024 | High | TR1 | Upkeep trigger removes counter; layer effect while counters remain |
| RE4 | Saddle N (tap creatures with total power ≥ N → Mount becomes creature) | TODO | OTJ 2024 | High | - | New permanent state + activated ability; similar to Crew |
| RE5 | Spree (modal spell where each mode has an additional cost) | TODO | OTJ 2024 | High | MD1 | Choose one or more modes, each requiring extra mana |
| RE6 | Gift (you may offer a gift; opponent chooses yes → you get bonus) | TODO | BLB 2024 | Medium | - | Political mechanic; target player decides |
| RE7 | Offspring (pay {1} extra → create 1/1 token copy with flying) | TODO | BLB 2024 | Low | R9 ✅ | Kicker-like; creates token on resolution |
| RE8 | Collect Evidence N (exile cards from GY with total MV ≥ N as cost) | TODO | MKM 2024 | Medium | - | Alternative cost using GY cards |
| RE9 | Cases (Enchantment subtype with "To solve" condition → bonus when solved) | TODO | MKM 2024 | High | SA1 | New card type behavior; needs solve tracking |
| RE10 | Rooms / Doors (double-faced enchantment — unlock second door for cost) | TODO | DSK 2024 | High | - | New DFC subtype; unlocking is activated ability |
| RE11 | Eerie (triggered when you cast an enchantment or enchanted permanent ETBs) | TODO | DSK 2024 | Low | TR1 | Trigger pattern; needs enchantment-cast event |
| RE12 | Expend N (condition: you spent N or more mana this turn) | TODO | DSK 2024 | Medium | - | Tracks total mana spent in turn |
| RE13 | Manifest Dread (reveal top 2, manifest one face-down) | TODO | DSK 2024 | High | CK14 | Variant of Manifest; needs face-down permanent state |
| RE14 | Committing a Crime (whenever you target an opponent's permanent/spell/player) | TODO | OTJ 2024 | Low | - | Trigger condition on targeting; needs targeting event |

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
| Ward MVP: checked at targeting time, not as triggered ability | Simplified vs CR 702.21 | K11.2 |
| No replacement effects (CR 614) | R11 TODO | R11 |
| No prevention effects (CR 615) | R12 TODO | R12 |
| Continuous effects only cover temporary P/T mods | No layers 1-6, no static auras/lords | P10.4 |
| No copy effects (CR 707) | Not implemented | P10.5 |
| No extra turns or extra phases (CR 500.7, 723) | Not implemented | P10.7 |
| No modal spells (choose one/two) | Not implemented | P10.8 |
| No fight mechanic (CR 701.14) | Implemented in P10.12 | DONE |
| No mana abilities vs non-mana abilities distinction | Partial (Effect::is_mana_ability) | P10.3 |
| Trigger system limited to 4 event types | ZoneChanged, StepStarted, AttackerDeclared, CombatEnded only | P10.9 |

---

### Phase 10: Missing Core Game Systems

Fundamental MTG game systems not yet tracked in any phase. Ordered by dependency.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P10.1 | Poison counters and 10-poison loss rule | DONE | 704.5c, 122.1b | Medium | - | SBA check: player with 10+ poison loses. Snapshot field already exists but no gameplay logic |
| P10.2 | -1/-1 counters | DONE | 122.1b | Medium | - | Counter type on creatures, affects P/T calculation |
| P10.3 | Mana abilities vs non-mana abilities formal distinction | TODO | 605 | Medium | - | Partial: Effect::is_mana_ability exists. Need: triggered mana abilities, mana ability timing rules during casting (CR 605.3a) |
| P10.4 | Full 7-layer continuous effects system | TODO | 613 | Very High | - | Currently only temporary P/T mods (layer 7c). Need all 7 layers + sublayers, timestamps, dependency system |
| P10.5 | Copy effects | TODO | 707 | Very High | P10.4 | Clone, Copy Enchantment, etc. Applies in layer 1. Requires copiable values concept |
| P10.6 | +1/+1 and -1/-1 counter annihilation SBA | DONE | 704.5q | Low | P10.2 | When both exist on a permanent, remove pairs. Already have +1/+1 counters |
| P10.7 | Extra turns and extra phases | TODO | 500.7, 723 | High | - | Time Walk, extra combat phases. Turn queue system |
| P10.8 | Modal spells (choose one / choose two) | TODO | 700.2 | High | - | Requires mode selection at cast time (CR 700.2a). Many common cards are modal |
| P10.9 | Expanded trigger event types | TODO | 603.1 | High | - | Currently only 4 types. Need: damage dealt, life gained/lost, spell cast, creature died, land played, counters placed, etc. |
| P10.10 | Bounce (return to hand) effect | DONE | 701.3 | Low | - | Unsummon-type effects. Need RulesAction::ReturnToHand |
| P10.11 | Exile-and-return (flicker) effects | TODO | - | Medium | - | Flickerwisp, Restoration Angel. Exile then return to battlefield. Needs delayed trigger system |
| P10.12 | Fight mechanic | DONE | 701.14 | Medium | - | Each creature deals damage equal to its power to the other. Common in green |
| P10.13 | Goad mechanic | TODO | 701.15 | Medium | - | Goaded creature must attack and must attack someone other than goader. Multiplayer-relevant |
| P10.14 | Proliferate | TODO | 701.34 | Medium | P10.1, P10.2 | Choose any number of permanents/players with counters, add one of each kind. Very common in recent sets |
| P15.6 | Bolster N | DONE | 701.39 | Low | - | Put N +1/+1 counters on the creature you control with the least toughness |
| P15.7 | Adapt N | DONE | 701.46 | Low | - | If this creature has no +1/+1 counters, put N +1/+1 counters on it |
| P10.15 | Investigate / Clue tokens | DONE | 701.16 | Low | R9 | Create predefined Clue artifact token with "{2}, Sacrifice: Draw a card" |
| P10.16 | Food tokens | DONE | - | Low | R9 | Predefined token: "{2}, {T}, Sacrifice: Gain 3 life" |
| P10.17 | Treasure tokens | DONE | - | Low | R9 | Predefined token: "{T}, Sacrifice: Add one mana of any color." Already very common |
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
| K11.1 | Defender | DONE | 702.3 | Low | - | "This creature can't attack." Functionally equivalent to CannotAttack but is the official keyword. Should be StaticAbility variant |
| K11.2 | Ward {N} | DONE | 702.21 | Medium | - | Simplified MVP: checked at targeting time. ward_cost field on CardDefinition; opponent must have enough mana |
| K11.3 | Prowess | TODO | 702.108 | Medium | P10.9 | Triggered: whenever you cast a noncreature spell, this gets +1/+1 until EOT. Needs "spell cast" trigger event |
| K11.4 | Protection from X | TODO | 702.16 | Very High | C6, R1 | DEBT: Damage prevented, Enchanting/Equipping falls off, Blocking prevented, Targeting prevented. Already tracked as K6 |
| K11.5 | Fear | DONE | 702.36 | Low | - | Can't be blocked except by artifact creatures and/or black creatures. Legacy keyword |
| K11.6 | Intimidate | TODO | 702.13 | Low | R14 | Can't be blocked except by artifact creatures and/or creatures sharing a color. Requires color identity checks |
| K11.7 | Skulk | DONE | 702.118 | Low | - | Can't be blocked by creatures with greater power |
| K11.8 | Shadow | DONE | 702.28 | Low | - | Can only block/be blocked by creatures with shadow |
| K11.9 | Horsemanship | DONE | 702.31 | Low | - | Can only block/be blocked by creatures with horsemanship (flying variant for Portal sets) |

**Tier 2: Common (appear in many sets)**

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| K11.10 | Undying | TODO | 702.93 | Medium | P10.2 | When dies with no +1/+1 counters, return with a +1/+1 counter. Needs -1/-1 interaction awareness |
| K11.11 | Persist | TODO | 702.79 | Medium | P10.2 | When dies with no -1/-1 counters, return with a -1/-1 counter. Needs -1/-1 counters |
| K11.12 | Wither | TODO | 702.80 | Medium | P10.2 | Damage dealt to creatures is dealt as -1/-1 counters instead |
| K11.13 | Infect | TODO | 702.90 | Medium | P10.1, P10.2 | Damage to creatures as -1/-1, damage to players as poison counters |
| K11.14 | Toxic N | DONE | 702.164 | Low | P10.1 | Whenever this deals combat damage to a player, that player gets N poison counters |
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
| K11.32 | Changeling | DONE | 702.73 | Low | R2 | Has all creature types. Simple flag on subtypes |
| K11.33 | Devoid | DONE | 702.114 | Low | R14 | Colorless regardless of mana cost. Characteristic-defining ability in layer 5 |

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
| P12.15 | Kindred card type | DONE | 308 | Low | R2 | "Kindred" type that has creature subtypes (e.g., Kindred Sorcery - Goblin). Renamed from "Tribal" |
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
| P15.1 | Surveil N | DONE | 701.25 | Low | - | Like scry but cards go to graveyard instead of bottom. Very common in recent sets |
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

### Phase 16: Sets, Formats & Collection Management

Card sets (expansions), format legality, and collection/deck management.

| # | Description | Status | CR Ref | Complexity | Dependency | Notes |
|---|-------------|--------|--------|------------|------------|-------|
| P16.1 | Card set/expansion data model | TODO | - | Medium | M6 | Set code, name, release date, set type (expansion, core, masters, etc.). MTGJSON provides this data |
| P16.2 | Card-to-set mapping (printings) | TODO | - | Medium | P16.1 | A card can appear in multiple sets. Collector number, rarity (common/uncommon/rare/mythic), art variations |
| P16.3 | Format definition system | TODO | - | Medium | - | Define formats: Standard, Pioneer, Modern, Legacy, Vintage, Pauper, Commander. Each format has rules (deck size, copies allowed, card pool) |
| P16.4 | Standard format (rotating sets) | TODO | - | Medium | P16.1, P16.3 | Last 2-3 years of sets. Rotation schedule. Banned list |
| P16.5 | Pioneer format | TODO | - | Low | P16.3 | Return to Ravnica forward. Banned list |
| P16.6 | Modern format | TODO | - | Low | P16.3 | 8th Edition forward. Banned list |
| P16.7 | Legacy format | TODO | - | Low | P16.3 | All sets. Banned list |
| P16.8 | Vintage format | TODO | - | Low | P16.3 | All sets. Restricted list (1 copy) + banned list |
| P16.9 | Pauper format | TODO | - | Low | P16.2, P16.3 | Only commons. Banned list |
| P16.10 | Commander format rules | TODO | - | High | P14.2, P16.3 | 100-card singleton, color identity, commander zone. Already tracked in P14.2 |
| P16.11 | Banned/restricted list system | TODO | - | Medium | P16.3 | Per-format lists. Source from MTGJSON or manual. Validate decks against them |
| P16.12 | Deck legality validation | TODO | - | Medium | P16.3, P16.11 | Check: correct deck size, max copies (4 or 1 for singleton), all cards legal in format, sideboard rules |
| P16.13 | Player collection (card inventory) | TODO | - | High | P16.2 | Track which cards a player owns. Quantities per printing. Import/export |
| P16.14 | Collection import from MTGJSON | TODO | - | Medium | P16.13, M6 | Import cards from MTGJSON AtomicCards.json with set/rarity data |
| P16.15 | Deck building with format validation | TODO | - | High | G4, P16.12, P16.13 | Build decks from collection, validate against format rules in real-time |
| P16.16 | Set browsing / card search | TODO | - | Medium | P16.1, P16.2 | Browse cards by set, search by name/type/text/color/rarity/format |
| P16.17 | Mana value (converted mana cost) | DONE | 202.3 | Low | - | mana_value() on ManaCost and CardDefinition; X counts as 0 for CMC |
| P16.18 | Rarity system | TODO | - | Low | P16.2 | Common, Uncommon, Rare, Mythic Rare, Special. Affects Pauper legality and draft |
| P16.19 | Draft / Sealed format support | TODO | - | Very High | P16.1, P16.2 | Open packs, pick cards, build deck. Needs booster pack generation |
| P16.20 | Sideboard support | TODO | - | Medium | P16.12 | 15-card sideboard. Swap between games in best-of-3. Format-dependent rules |

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
