# CLIPS Rules Engine Integration — Design Specification

## Overview

Echomancy adopts the same architecture as MTG Arena's Game Rules Engine (GRE): a **Rust core engine** handles fundamental Magic rules (turns, priority, stack, zones), while **CLIPS** (an expert system) handles card-specific behavior via forward-chaining rules.

This decouples the core engine from individual cards. The engine doesn't know what Lightning Bolt does — it just announces "spell resolving" and CLIPS rules determine the effects.

**Key principle: CLIPS extends Rust, it does not replace it.** The existing Rust engine (665+ tests) handles core mechanics. CLIPS fills the gaps where card-specific behavior is currently hardcoded or missing (resolve_spell, triggered abilities, continuous effects).

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

### What Stays in Rust (existing, tested, working)

- Turn/step machine (12 steps, 5 phases)
- Priority system and passing
- Stack push/pop/resolve orchestration
- Zone transitions (hand → battlefield → graveyard)
- Mana pool management (add/spend/clear)
- Combat declaration validation
- Mana payment auto-pay algorithm
- Game lifecycle (create, add players, start, finish)
- Specifications (can_play_land, can_cast_spell, etc.)
- State-based action detection (lethal damage, 0 life)

### What CLIPS Handles (new, fills current gaps)

- **Spell effects**: what happens when a spell resolves (damage, draw, buff, destroy)
- **Triggered ability effects**: what happens on ETB, death, attack, etc.
- **Continuous effects**: lords, enchantments, "creatures you control get +1/+1"
- **Replacement effects**: "if this would be destroyed, instead..."
- **Card-specific keywords**: beyond the basic Flying/Reach/Vigilance/Haste

### Integration Points (where Rust calls CLIPS)

CLIPS is invoked at specific points where card-specific behavior is needed:

1. **`resolve_spell()`** → currently quasi no-op → CLIPS determines spell effects
2. **`execute_triggered_abilities()`** → currently empty loop → CLIPS executes triggers
3. **`enter_battlefield()`** → CLIPS checks for ETB triggers
4. **`move_permanent_to_graveyard()`** → CLIPS checks for death triggers
5. **`on_enter_step()`** → CLIPS checks "at the beginning of..." triggers
6. **Continuous effects** → new subsystem, fully in CLIPS

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
4. Run(env, MAX_RULES)                 — CLIPS rules fire (bounded, not -1)
5. Check: awaiting-input facts?
   YES → read choice-point, prompt player, assert choice, goto 4
   NO  → continue
6. Check: rules_fired == MAX_RULES?
   YES → possible infinite loop, log error, abort
   NO  → continue
7. Collect all action-* facts          — output orders
8. Sort by priority slot
9. Validate each action in Rust        — CLIPS proposes, Rust validates
10. Execute valid actions as Game mutations
11. (Next event: goto 1)
```

We use the **full reset cycle** (steps 1-2) for simplicity. This eliminates stale fact bugs at the cost of re-asserting all state each cycle. For typical MTG board sizes (<100 facts), this costs ~140-250us — negligible.

## Layered Architecture

```
echomancy-core/
  domain/
    game/                ← Game aggregate (STAYS — existing tested code)
      mod.rs             ← Game struct, apply() dispatcher
      accessors.rs       ← read-only queries
      internals.rs       ← mutation helpers
      automation.rs      ← auto-advance, auto-resolve
      play_land.rs       ← handler (STAYS)
      cast_spell.rs      ← handler (STAYS, calls RulesEngine at resolve)
      ...
    rules_engine.rs      ← NEW: trait RulesEngine (domain interface)
  infrastructure/
    clips/               ← NEW: CLIPS implementation of RulesEngine
      mod.rs             ← ClipsEngine struct (safe wrapper)
      bridge.rs          ← Game state → CLIPS facts serialization
      actions.rs         ← CLIPS action-* facts → GameAction enum
      router.rs          ← CLIPS I/O capture → Rust tracing
    game_state_export.rs ← (existing)
    game_snapshot.rs     ← (existing)

clips-sys/               ← Raw C FFI bindings (exists from PoC)
```

### Domain Trait (technology-agnostic)

```rust
// domain/rules_engine.rs
pub trait RulesEngine {
    fn evaluate(
        &mut self,
        state: &Game,
        event: &GameEvent,
    ) -> Result<RulesResult, RulesError>;

    fn resume_after_choice(
        &mut self,
        choice: &PlayerChoice,
    ) -> Result<RulesResult, RulesError>;
}

pub struct RulesResult {
    pub actions: Vec<GameAction>,
    pub awaiting_input: Option<InputRequest>,
    pub rules_fired: i64,
    pub warnings: Vec<String>,
}
```

### Infrastructure Implementation

```rust
// infrastructure/clips/mod.rs
pub struct ClipsEngine {
    env: *mut c_void,
    error_buffer: Vec<String>,
    trace_buffer: Vec<String>,
    max_rules_per_cycle: i64,
}

impl RulesEngine for ClipsEngine { ... }
```

This way the domain never knows about CLIPS. If we ever swap it for something else, only the infrastructure changes.

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

Action orders produced by card rules. Separate template per action type, prefixed with `action-`:

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

Rust collects all `action-*` facts after Run() and sorts by the `priority` slot:

| Priority Range | Category |
|----------------|----------|
| 0-99 | State-based actions |
| 100-199 | Triggered abilities |
| 200-299 | Replacement effects |
| 300+ | Player choice required |

## Error Handling and Debugging

### Core Principle: CLIPS Proposes, Rust Validates

CLIPS can never corrupt game state directly. The flow is:
1. CLIPS rules fire and assert `action-*` facts
2. Rust reads these facts
3. Rust validates each action against the domain model
4. Only valid actions are applied
5. Invalid actions are logged and discarded

### CLIPS Router System

CLIPS has a router system for I/O redirection. We register a custom router that captures all output:

- `"stderr"` → Rust `tracing::error!()`
- `"stdwrn"` → Rust `tracing::warn!()`
- `"stdout"` → Rust `tracing::debug!()` (watch traces)

### Debugging Facilities

- `Watch(env, FACTS)` — trace every assert/retract
- `Watch(env, RULES)` — trace every rule firing
- `Watch(env, ACTIVATIONS)` — trace agenda changes
- Enabled conditionally (debug builds or log level)

### Error Scenarios and Mitigations

| Scenario | Detection | Prevention | Recovery |
|----------|-----------|------------|----------|
| Rule syntax error | `Build()` returns error + router capture | Validate at load time | Reject card, log error |
| Infinite loop | `Run(N)` returns N (hit limit) | Bounded execution, never Run(-1) | Abort cycle, log trace |
| Invalid action data | Rust validation after collecting actions | CLIPS slot type constraints | Ignore action, log warning |
| Unexpected fact retraction | Router traces retractions | Modules restrict access | Snapshot before/after comparison |

### Rule Validation

- Use `Build()` per-construct (not `Load()` for whole files) for per-rule error feedback
- Validate in a throwaway CLIPS environment at startup
- Capture router output for specific error messages
- No static analysis tool exists for CLIPS — validation is runtime

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
    state-based-actions.clp
    combat-damage.clp
    mana-abilities.clp
    stack-resolution.clp
  cards/                             # Card-specific rules — loaded per-game
    a/
    b/
    ...
    e/
      elvish-visionary.clp
    g/
      glorious-anthem.clp
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

~25% of all MTG cards need no .clp file at all (vanilla + keyword-only).

## Card Rule Examples

### Lightning Bolt (Instant — deal 3 damage)

```clips
(defmodule CARD-LIGHTNING-BOLT (import CORE deftemplate ?ALL))

(defrule CARD-LIGHTNING-BOLT::resolve
  "Lightning Bolt deals 3 damage to any target."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id))
  (stack-item (id ?spell-id) (card-id "lightning-bolt")
              (status RESOLVING) (target ?target))
  =>
  (assert (action-damage (source ?spell-id) (target ?target) (amount 3))))
```

### Elvish Visionary (Creature — ETB: draw a card)

```clips
(defmodule CARD-ELVISH-VISIONARY (import CORE deftemplate ?ALL))

(defrule CARD-ELVISH-VISIONARY::etb-draw
  "When Elvish Visionary enters the battlefield, draw a card."
  (game-event (type ZONE_CHANGED) (source-id ?id) (data "ENTERED_BATTLEFIELD"))
  (permanent (instance-id ?id) (card-id "elvish-visionary") (controller ?player))
  =>
  (assert (action-draw (player ?player) (amount 1))))
```

### Glorious Anthem (Enchantment — static +1/+1)

```clips
(defmodule CARD-GLORIOUS-ANTHEM (import CORE deftemplate ?ALL))

(defrule CARD-GLORIOUS-ANTHEM::buff-creatures
  "Creatures you control get +1/+1."
  (permanent (instance-id ?anthem-id) (card-id "glorious-anthem")
             (controller ?player) (zone battlefield))
  (permanent (instance-id ?creature-id&~?anthem-id)
             (controller ?player) (zone battlefield) (card-type creature))
  =>
  (assert (continuous-effect (source ?anthem-id) (target ?creature-id)
                             (power-mod 1) (toughness-mod 1))))
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
1. `Run(N)` returns to Rust (because of `(halt)`)
2. Rust reads `awaiting-input` → prompts player via UI
3. Player makes choice
4. Rust asserts `(sacrifice-choice (player "p1") (chosen "creature-42"))`
5. Rust calls `Run(N)` again
6. Rules resume firing

## Testing Strategy

### Integration Tests (Rust ↔ CLIPS)

Test the full cycle for each card:

```rust
#[test]
fn lightning_bolt_deals_3_damage_to_player() {
    let mut engine = ClipsEngine::new(CORE_RULES);
    engine.load_card_rules("lightning-bolt", LIGHTNING_BOLT_RULES);

    let game = make_game_with_bolt_on_stack(target: "p2");
    let result = engine.evaluate(&game, &GameEvent::SpellResolving("bolt-1"));

    assert_eq!(result.actions.len(), 1);
    assert!(matches!(result.actions[0],
        GameAction::DealDamage { target, amount: 3, .. } if target == "p2"));
}
```

### Testing Levels

| Level | What | How |
|-------|------|-----|
| CLIPS rule syntax | Each .clp file loads without errors | `Build()` in throwaway env |
| Card behavior | Each card produces correct actions | Integration test per card |
| Rule interactions | Multiple cards interacting | Integration test with complex board |
| Full game flow | End-to-end with CLIPS | Existing game tests + CLIPS engine |

## Migration Roadmap

CLIPS does NOT replace existing Rust code. It extends it at the integration points.

| Phase | What | Touches existing code? |
|-------|------|----------------------|
| **M1** | ClipsEngine safe wrapper + router + tests | No |
| **M2** | Bridge: serialize Game state → CLIPS facts | No |
| **M3** | Connect `resolve_spell()` to CLIPS | Minimal — add RulesEngine call |
| **M4** | Connect `execute_triggered_abilities()` to CLIPS | Minimal — add RulesEngine call |
| **M5** | MTGJSON card data loader (CardDefinition from JSON) | No |
| **M6** | Continuous effects / 7-layer system in CLIPS | New subsystem |
| **M7** | MTGJSON Oracle text → .clp auto-generation (future) | No |

## Performance Budget

Based on PoC measurements (Apple Silicon, debug build):

| Operation | Time |
|-----------|------|
| CLIPS Environment create + load rules | ~200-400 µs |
| Full state assert (100 facts) | ~140-250 µs |
| Rule execution (Run) | ~7-11 µs |
| Single fact assert | ~1.4-2.5 µs |
| Total per-event cycle | ~350-700 µs |

At 60fps, one frame is ~16,700 µs. A full CLIPS cycle fits comfortably within a single frame.

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
| Error handling | CLIPS proposes, Rust validates | CLIPS bugs never corrupt game state |
| Debugging | Custom router → Rust tracing | Full observability of CLIPS internals |
| Execution | Bounded `Run(N)`, never `Run(-1)` | Prevents infinite loops |
| Architecture | Domain trait `RulesEngine`, infra impl | Domain stays technology-agnostic |
