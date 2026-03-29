# CLIPS Rules Engine Integration — Design Specification

## Overview

Echomancy adopts the same architecture as MTG Arena's Game Rules Engine (GRE): a **Rust core engine** handles fundamental Magic rules (turns, priority, stack, zones), while **CLIPS** (an expert system) handles card-specific behavior via forward-chaining rules.

This decouples the core engine from individual cards. The engine doesn't know what Lightning Bolt does — it just announces "spell resolving" and CLIPS rules determine the effects.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Echomancy Engine                         │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │  Rust Core    │───▶│  CLIPS Rules  │───▶│  Rust Applies  │ │
│  │  (Game, turns,│    │  (card-specific│    │  (mutations to │ │
│  │   priority,   │    │   behavior,    │    │   Game state)  │ │
│  │   stack,      │    │   triggers,    │    │               │ │
│  │   zones)      │    │   effects)     │    │               │ │
│  └──────────────┘    └──────────────┘    └───────────────┘  │
│         │                   ▲  │                   ▲         │
│         │    assert facts   │  │   action-* facts  │         │
│         └───────────────────┘  └───────────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### Inspired by MTG Arena

Arena uses C++ + CLIPS with a "whiteboard + nap" pattern:

1. The GRE writes game events to a shared "whiteboard" (CLIPS working memory)
2. The GRE "takes a nap" (yields control)
3. CLIPS rules fire — card-specific rules modify the whiteboard
4. The GRE "wakes up" and executes what remains

We replicate this with Rust + CLIPS via C FFI.

## The Cycle

Each time a game event occurs (spell resolves, creature enters battlefield, step starts, etc.):

```
1. Reset(env)                          — clean slate
2. Assert ALL current game state       — input facts (deftemplates)
3. Assert the event                    — transient event fact
4. Run(-1)                             — CLIPS rules fire to fixed point
5. Check: awaiting-input facts?
   YES → read choice-point, prompt player, assert choice, goto 4
   NO  → continue
6. Collect all action-* facts          — output orders
7. Sort by priority slot
8. Execute actions as Game mutations   — Rust applies changes
9. (Next event: goto 1)
```

We start with the **full reset cycle** (steps 1-2) for simplicity. This eliminates stale fact bugs at the cost of re-asserting all state each cycle. For typical MTG board sizes (<100 facts), this costs ~140-250us — negligible.

If profiling shows performance issues, we can switch to **persistent mirroring with delta updates** (retract/assert only changed facts).

## CLIPS Fact Schema

### Input Facts (Rust → CLIPS)

Game state represented as deftemplates with named slots:

```clips
(deftemplate player
  (slot id (type STRING))
  (slot life (type INTEGER))
  (slot is-active (type SYMBOL) (allowed-symbols TRUE FALSE))
  (slot has-priority (type SYMBOL) (allowed-symbols TRUE FALSE)))

(deftemplate permanent
  (slot instance-id (type STRING))
  (slot card-id (type STRING))
  (slot card-name (type STRING))
  (slot controller (type STRING))
  (slot owner (type STRING))
  (slot zone (type SYMBOL) (default battlefield))
  (slot card-type (type SYMBOL))
  (slot tapped (type SYMBOL) (allowed-symbols TRUE FALSE) (default FALSE))
  (slot summoning-sick (type SYMBOL) (allowed-symbols TRUE FALSE) (default FALSE))
  (slot power (type INTEGER) (default 0))
  (slot toughness (type INTEGER) (default 0))
  (slot damage (type INTEGER) (default 0))
  (multislot keywords)
  (multislot counters))

(deftemplate mana-pool
  (slot player-id (type STRING))
  (slot white (type INTEGER) (default 0))
  (slot blue (type INTEGER) (default 0))
  (slot black (type INTEGER) (default 0))
  (slot red (type INTEGER) (default 0))
  (slot green (type INTEGER) (default 0))
  (slot colorless (type INTEGER) (default 0)))

(deftemplate stack-item
  (slot id (type STRING))
  (slot card-id (type STRING))
  (slot controller (type STRING))
  (slot status (type SYMBOL))
  (slot target (type STRING)))

(deftemplate turn-state
  (slot current-step (type SYMBOL))
  (slot active-player (type STRING))
  (slot turn-number (type INTEGER)))

(deftemplate game-event
  (slot type (type SYMBOL))
  (slot source-id (type STRING))
  (slot controller (type STRING))
  (slot target-id (type STRING))
  (slot data (type STRING)))

(deftemplate attached
  (slot enchantment-id (type STRING))
  (slot target-id (type STRING)))
```

### Output Facts (CLIPS → Rust)

Action orders produced by card rules. Separate template per action type:

```clips
(deftemplate action-draw
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-damage
  (slot priority (type INTEGER) (default 100))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-destroy
  (slot priority (type INTEGER) (default 100))
  (slot target (type STRING)))

(deftemplate action-gain-life
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-lose-life
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-move-zone
  (slot priority (type INTEGER) (default 100))
  (slot card-id (type STRING))
  (slot from-zone (type SYMBOL))
  (slot to-zone (type SYMBOL)))

(deftemplate action-add-mana
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot color (type SYMBOL))
  (slot amount (type INTEGER)))

(deftemplate action-tap
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING)))

(deftemplate action-untap
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING)))

(deftemplate action-add-counter
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING))
  (slot counter-type (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-create-token
  (slot priority (type INTEGER) (default 100))
  (slot controller (type STRING))
  (slot name (type STRING))
  (slot power (type INTEGER))
  (slot toughness (type INTEGER))
  (multislot types)
  (multislot keywords))
```

### Signal Facts

```clips
(deftemplate awaiting-input
  (slot type (type SYMBOL))
  (slot player (type STRING))
  (slot prompt (type STRING)))
```

### Priority Ordering

Rust collects all `action-*` facts after `Run(-1)` and sorts by the `priority` slot:

| Priority Range | Category |
|----------------|----------|
| 0-99 | State-based actions |
| 100-199 | Triggered abilities |
| 200-299 | Replacement effects |
| 300+ | Player choice required |

## File Organization

### Directory Structure

```
rules/
  core/                              # Generic MTG rules — embedded in binary
    templates.clp                    # All deftemplate definitions
    flying.clp                       # "Can only be blocked by Flying/Reach"
    reach.clp
    vigilance.clp
    haste.clp
    summoning-sickness.clp
    state-based-actions.clp          # SBA rules (lethal damage, 0 life, etc.)
    combat-damage.clp
    mana-abilities.clp
    stack-resolution.clp
  cards/                             # Card-specific rules — loaded per-game
    a/
      acidic-slime.clp
    b/
      birds-of-paradise.clp
    e/
      elvish-visionary.clp
    g/
      glorious-anthem.clp
      grizzly-bears.clp              # (empty or nonexistent — vanilla)
    l/
      lightning-bolt.clp
    ...
```

### Loading Strategy

- **Core rules**: embedded in binary via `include_str!()`. Always loaded.
- **Card rules**: loaded from filesystem at game start. Only load rules for cards in the current game (~120 files for 2 decks of 60).
- **CLIPS modules**: each card file declares `(defmodule CARD-<NAME>)` to avoid rule name collisions.

### Which Cards Need a .clp File?

| Card Type | Needs .clp? | Example |
|-----------|-------------|---------|
| Vanilla creature (no text) | No | Grizzly Bears |
| Keyword-only creature | No — generic rules | Serra Angel (Flying, Vigilance) |
| Creature with triggered ability | Yes | Elvish Visionary (ETB: draw) |
| Instant/Sorcery with effect | Yes | Lightning Bolt (3 damage) |
| Enchantment with static ability | Yes | Glorious Anthem (+1/+1 lord) |
| Planeswalker | Yes | (complex, future) |

Approximately 25% of all MTG cards need no .clp file at all (vanilla + keyword-only).

## Card Rule Examples

### Lightning Bolt (Instant — deal 3 damage)

```clips
(defmodule CARD-LIGHTNING-BOLT (import CORE deftemplate ?ALL))

(defrule CARD-LIGHTNING-BOLT::resolve
  "Lightning Bolt deals 3 damage to any target."
  (game-event (type SPELL_RESOLVING)
              (source-id ?spell-id))
  (stack-item (id ?spell-id)
              (card-id "lightning-bolt")
              (status RESOLVING)
              (target ?target))
  =>
  (assert (action-damage (source ?spell-id) (target ?target) (amount 3))))
```

### Elvish Visionary (Creature — ETB: draw a card)

```clips
(defmodule CARD-ELVISH-VISIONARY (import CORE deftemplate ?ALL))

(defrule CARD-ELVISH-VISIONARY::etb-draw
  "When Elvish Visionary enters the battlefield, draw a card."
  (game-event (type ZONE_CHANGED)
              (source-id ?id)
              (data "ENTERED_BATTLEFIELD"))
  (permanent (instance-id ?id)
             (card-id "elvish-visionary")
             (controller ?player))
  =>
  (assert (action-draw (player ?player) (amount 1))))
```

### Glorious Anthem (Enchantment — static +1/+1)

```clips
(defmodule CARD-GLORIOUS-ANTHEM (import CORE deftemplate ?ALL))

(defrule CARD-GLORIOUS-ANTHEM::buff-creatures
  "Creatures you control get +1/+1."
  (permanent (instance-id ?anthem-id)
             (card-id "glorious-anthem")
             (controller ?player)
             (zone battlefield))
  (permanent (instance-id ?creature-id&~?anthem-id)
             (controller ?player)
             (zone battlefield)
             (card-type creature))
  =>
  (assert (continuous-effect
            (source ?anthem-id)
            (target ?creature-id)
            (power-mod 1)
            (toughness-mod 1))))
```

## Choice Points

When a CLIPS rule needs player input:

```clips
(defrule some-card::needs-sacrifice
  (sacrifice-required (player ?p) (count ?n))
  (not (sacrifice-choice (player ?p)))
  =>
  (assert (awaiting-input (type sacrifice) (player ?p)
                          (prompt "Choose a creature to sacrifice")))
  (halt))
```

Flow:
1. `Run(-1)` returns to Rust (because of `(halt)`)
2. Rust reads `awaiting-input` → prompts player via UI
3. Player makes choice
4. Rust asserts `(sacrifice-choice (player "p1") (chosen "creature-42"))`
5. Rust calls `Run(-1)` again
6. Rules resume firing

## Integration with Rust Game Aggregate

### Where CLIPS Sits

```
echomancy-core/
  domain/
    game/           ← Game::apply() dispatches to handlers
      mod.rs
      handlers/     ← play_land, cast_spell, etc.
    clips/          ← NEW: CLIPS integration module
      mod.rs        ← ClipsEngine wrapper (safe Rust over FFI)
      bridge.rs     ← Game state → CLIPS facts serialization
      actions.rs    ← CLIPS action-* facts → Game mutations
  infrastructure/
    ...

clips-sys/          ← Raw C FFI bindings (already exists from PoC)
```

### When CLIPS Is Invoked

The `Game::apply()` handler calls the CLIPS engine at specific points:

1. **Spell resolving** → CLIPS determines spell effects
2. **Permanent enters battlefield** → CLIPS checks for ETB triggers
3. **Permanent leaves battlefield** → CLIPS checks for LTB triggers
4. **Step/phase starts** → CLIPS checks for "at the beginning of..." triggers
5. **Combat events** → CLIPS checks for attack/block triggers
6. **State-based actions** → CLIPS evaluates SBA rules
7. **Continuous effects** → CLIPS evaluates static abilities (layer system, future)

## Performance Budget

Based on PoC measurements (Apple Silicon, debug build):

| Operation | Time |
|-----------|------|
| CLIPS Environment create + load rules | ~200-400 µs |
| Full state assert (100 facts) | ~140-250 µs |
| Rule execution (Run) | ~7-11 µs |
| Single fact assert | ~1.4-2.5 µs |
| Total per-event cycle | ~350-700 µs |

At 60fps, one frame is ~16,700 µs. A full CLIPS cycle fits comfortably within a single frame, even with multiple events per frame.

## MTGJSON Integration (Future)

1. Download MTGJSON card database (Oracle text, keywords, types, costs)
2. Auto-generate `.clp` files for templated cards (~25-30% of cards)
3. Manually write `.clp` files for complex cards
4. The `CardDefinition` in Rust holds data (name, cost, types, P/T, keywords)
5. CLIPS holds behavior (what happens when the card is played, triggered, etc.)

## Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rules engine | CLIPS 6.4.2 (C, compiled from source) | Proven by Arena, Rete algorithm, forward-chaining |
| Rust bindings | Custom clips-sys crate (minimal FFI) | Existing bindings abandoned, we maintain our own |
| Fact model | Deftemplates (named slots) | Self-documenting, extensible, type-safe |
| State sync | Full reset cycle | Simple, no stale facts, performance acceptable |
| File organization | One .clp per card, alphabetical dirs | Proven at scale by Forge (20K+) and XMage (28K+) |
| Core rules | Embedded in binary | Always available, rarely change |
| Card rules | Loaded from filesystem per-game | Editable without recompile, selective loading |
| Choice points | `(halt)` in CLIPS rules | Native CLIPS mechanism for pausing execution |
