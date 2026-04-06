# LS1: Layer System (CR 613)

## Overview

The Layer System is the rules framework that determines how continuous effects interact when
multiple effects modify the same characteristic of a permanent simultaneously. MTG Comprehensive
Rules section 613 defines seven ordered layers, each handling a distinct category of modification.
Sublayers within Layer 7 handle power/toughness changes in a specific sequence.

The engine currently applies all continuous effects in insertion order, which produces incorrect
results whenever two effects touch the same characteristic. This spec replaces that ad-hoc
summation with a standards-compliant evaluation pipeline.

**Design goal**: Any query for a permanent's effective characteristics — power, toughness, types,
colors, abilities — must return the value that the official CR would produce, regardless of the
order in which the affecting cards entered the battlefield or their spells resolved.

**Relationship to other systems**:
- Replaces the current flat sum in `current_power()` / `current_toughness()`.
- Extends the existing `ContinuousEffect` data model with a `layer` discriminant.
- Future specs for enchantment auras, equipment, and copy effects will add new effect variants
  that slot into the appropriate layer automatically.

---

## User Stories

**As a player**, I want a +3/+3 effect from Giant Growth to interact correctly with a "becomes a
1/1" effect from a Humility-like card, so that the final P/T is what the rules say it should be
(1/1 from layer 7b set, then +3/+3 from layer 7c pump, giving 4/4) rather than an arbitrary
result depending on which effect was registered first.

**As a developer**, I want to add a new continuous effect to a card definition and have it
automatically slot into the correct evaluation layer without manually adjusting existing
calculation code.

**As a QA validator**, I want unit tests that prove each layer applies in the right order with
multiple overlapping effects active simultaneously.

---

## Player Experience

The layer system is invisible infrastructure — players never interact with it directly. Its
correctness is visible through:

- Correct P/T displayed on creatures when multiple spells or permanents modify them.
- Correct card types displayed when type-changing effects are active.
- Correct color indicators when color-changing effects are active.
- Correct keyword abilities displayed when ability-adding or ability-removing effects are active.

There is no UI flow change from this spec. All changes are in the domain query layer.

---

## Game Rules and Mechanics

### The Seven Layers

Effects are applied in strict layer order. Within each layer, effects apply in timestamp order
(oldest effect first), except where dependency rules require otherwise (see Dependency Rules below).

| Layer | Handles | MTG CR |
|-------|---------|--------|
| 1 | Copy effects | 613.2 |
| 2 | Control-changing effects | 613.3 |
| 3 | Text-changing effects | 613.4 |
| 4 | Type-changing effects (card type, subtype, supertype) | 613.1d |
| 5 | Color-changing effects | 613.1e |
| 6 | Ability-adding and ability-removing effects | 613.1f |
| 7 | Power/toughness-changing effects (sublayers a–d) | 613.1g |

Layers 1–3 are out of scope for this spec (see Out of Scope). Layers 4–7 are in scope.

### Layer 4: Type-Changing Effects

Effects that grant, remove, or replace card types, supertypes (Legendary, Snow, World, Basic),
and subtypes.

**Examples**:
- "Until end of turn, target land becomes a 3/3 creature" — adds Creature type.
- "All creatures lose all creature types and become Illusions" — replaces subtypes.
- "CARDNAME is also an artifact in addition to its other types" — adds Artifact type.

The query `effective_types(permanent)` must apply all Layer 4 effects in timestamp order and
return the resulting set of types and subtypes. Per CR 613.3, within Layer 4 any
Characteristic-Defining Abilities (CDAs) that affect subtypes apply before timestamp-ordered
effects. Note: CDAs can only affect subtypes (e.g. Changeling — "is every creature type") —
they cannot define card types (Creature, Artifact, Land, etc.). A CDA that reads "is every
creature type" changes the subtype set, not the card type line.

### Layer 5: Color-Changing Effects

Effects that change, add, or remove colors.

**Examples**:
- "All permanents are blue" — sets color to blue.
- "CARDNAME is also green" — adds green.

The query `effective_colors(permanent)` must apply all Layer 5 effects in timestamp order and
return the resulting color set. Per CR 613.3, CDAs that affect color apply before
timestamp-ordered effects within Layer 5. A card like Transguild Courier ("CARDNAME is all
colors") has its color defined by a CDA evaluated first, and any timestamp-ordered color-changing
effects are applied on top.

### Layer 6: Ability-Adding and Ability-Removing Effects

Effects that grant keyword abilities (Flying, Trample, etc.) or remove them (e.g., "Creatures
lose flying"). This layer also handles keyword counters (flying counter, trample counter, etc.,
introduced in Ikoria) — a permanent bearing a keyword counter has that keyword granted in Layer 6,
and the `effective_abilities` query must include abilities from keyword counters alongside those
from Layer 6 effect records.

**CantHaveAbility effects**: Some effects state that a permanent or class of permanents cannot
have a given ability (e.g., "Creatures can't have hexproof"). These are modeled as a
`CantHaveAbility` variant in the Layer 6 data model. A `CantHaveAbility` effect overrides any
"gains ability" effect for that ability regardless of timestamp (CR 613.1f). That is, if both a
"gains Hexproof" effect and a "can't have Hexproof" effect are active on the same permanent,
the permanent does not have Hexproof.

**CDA restriction**: Per CR 604.3a, CDAs can only define colors, subtypes, power, or toughness.
CDAs cannot define keyword abilities or card types. Therefore no CDA applies in Layer 6 — the
CDA ordering exception from CR 613.3 does not have any Layer 6 cases in a correct rules
implementation.

**Examples**:
- "Creatures you control gain trample until end of turn."
- "All creatures lose all abilities."
- "Creatures can't have hexproof." — modeled as `CantHaveAbility(Hexproof)`.

The query `effective_abilities(permanent)` must apply all Layer 6 effects and return the
resulting keyword set. Abilities granted by Layer 6 effects are not part of `CardDefinition`
and must not be stored on the definition — they exist only in the `ContinuousEffect` record or
as keyword counters on the `PermanentState`.

### Layer 7: Power/Toughness-Changing Effects

Layer 7 is subdivided into exactly four mandatory sublayers applied in this exact order (CR 613.4):

| Sublayer | What applies | Examples |
|----------|-------------|---------|
| 7a | Characteristic-Defining Abilities (CDAs) that define P/T | "CARDNAME's power and toughness are each equal to the number of cards in your hand" |
| 7b | Effects that set P/T to a specific value | "CARDNAME is a 2/2", Humility, Godhead of Awe |
| 7c | All other P/T modifications — including +1/+1 counters, -1/-1 counters, and pump/anthem effects | Giant Growth (+3/+3), Glorious Anthem (+1/+1 to all), one +1/+1 counter = +1/+1, one -1/-1 counter = -1/-1 |
| 7d | Switch power and toughness effects | "Switch CARDNAME's power and toughness" |

All counter-based modifications (both +1/+1 and -1/-1) reside in 7c and are summed together with
all other non-setting P/T modifications in that sublayer. After all four sublayers, the result is
the effective P/T. Queries `effective_power(permanent)` and `effective_toughness(permanent)`
replace the existing `current_power()` / `current_toughness()` methods with layer-aware
equivalents.

**Post-Layer-4 types for Layer 7 eligibility**: Layer 7 P/T effects must check whether a
permanent is a creature using its post-Layer-4 effective types, not its card definition types.
A land animated to a creature in Layer 4 participates in Layer 7 P/T calculations. A creature
type-changed to a non-creature in Layer 4 does not receive creature-only Layer 7 effects in that
calculation pass.

**Critical correctness example**:
- Creature is 2/2 (base from card definition).
- Humility-like "All creatures have base P/T 1/1" effect active (Layer 7b sets to 1/1).
- Giant Growth cast (+3/+3, Layer 7c).
- Without layer system: result depends on insertion order.
- With layer system: 7b sets to 1/1, then 7c adds +3/+3 → 4/4. Always correct.

**Layer 7d correctness examples**:
- A 1/3 creature with a Layer 7c +0/+1 effect and a Layer 7d switch effect:
  after 7c P/T = 1/4; after 7d switch → `effective_power` = 4, `effective_toughness` = 1.
- A 2/4 creature with a Layer 7b "becomes 3/5" effect and a Layer 7d switch effect:
  after 7b P/T = 3/5; after 7d switch → `effective_power` = 5, `effective_toughness` = 3.

**Counter example**:
- Creature is 4/4 (base from card definition).
- One +1/+1 counter and one -1/-1 counter both in Layer 7c.
- Net from counters in 7c: +0/+0 (they sum together).
- Effective P/T: 4/4.
- Note: State-Based Action 704.5q removes a +1/+1 counter and a -1/-1 counter from the same
  permanent as a pair before the layer system is queried in normal gameplay, so this case is
  primarily relevant in contrived test scenarios.

### Characteristic-Defining Abilities (CDAs)

Some cards have P/T defined by a rule on the card itself rather than a printed number. For
example: "CARDNAME's power and toughness are each equal to the number of cards in your hand."
These are Characteristic-Defining Abilities (CDAs) and apply in Layer 7a.

CDAs are evaluated at the time the layer system computes Layer 7a, not at the time the card
entered the battlefield. Their value is dynamic and may change as game state changes.

Per CR 604.3a, CDAs can only define: colors, subtypes, power, or toughness. They cannot define
card types or keyword abilities. The CDA ordering exception (CR 613.3) therefore applies only
within Layers 4 (subtype CDAs), 5 (color CDAs), and 7a (P/T CDAs).

For this spec, CDA support in Layer 7a is limited to: the CDA is modeled as a `SetPowerToughness`
effect variant that stores a `PtFormula` enum. The only required `PtFormula` variant for MVP is
`CardsInControllerHand`, evaluated at query time from game state. Future variants (devotion,
number of creatures, etc.) are out of scope.

The effect data model must include an `is_cda: bool` flag (or equivalent discriminant) so the
engine can identify which effects are CDAs and apply them before timestamp-ordered effects in
layers where CDA ordering is required.

### Continuous Effect Data Model

The existing `ContinuousEffect` struct models only P/T modifiers. It must be extended to cover
all in-scope layers.

A continuous effect record must carry:

| Field | Purpose |
|-------|---------|
| Layer discriminant | Which layer this effect belongs to (4, 5, 6, or 7 with sublayer) |
| Effect payload | What the effect does (type set, color grant, ability grant or remove, `CantHaveAbility`, P/T delta, etc.) |
| Duration | When the effect expires |
| Timestamp | Monotonically increasing integer assigned at effect creation (see Timestamp Model below) |
| `source_id` | ID of the source that generated this effect (see Source ID below) |
| Controller ID | Player who controlled the source when the effect was created |
| `is_cda` | Whether this effect originates from a Characteristic-Defining Ability |
| Target scope | Either a locked set of permanent IDs or a continuous filter (see Target Scope below) |

#### Source ID

The field is named `source_id`, not `source_permanent_id`. This is because effects from spells
(e.g. Giant Growth, "all creatures get +1/+1 until end of turn") have no source permanent after
the spell resolves and leaves the stack. For static-ability-generated effects, `source_id` is the
ID of the source permanent on the battlefield. For spell-resolution effects, `source_id` is the
card instance ID that was on the stack when the effect was created. Both cases produce a valid
identifier for dependency tracking.

#### Target Scope

The target scope determines which permanents the effect applies to:

- `LockedSet(Vec<PermanentId>)`: The set of affected permanents is fixed at the moment the effect
  is created and never changes. Used for effects generated by spell resolution (CR 611.2c):
  "all creatures get +1/+1 until end of turn" locks the set of creatures at resolution time; a
  creature entering the battlefield after resolution is NOT affected.
- `Filter(EffectFilter)`: The set of affected permanents is re-evaluated each time the layer
  system queries this effect. Used for effects generated by static abilities (e.g. Glorious
  Anthem — "creatures you control get +1/+1"): a creature entering the battlefield after the
  anthem came into play IS affected immediately.

The distinction between `LockedSet` and `Filter` is mandatory for rules correctness. Without it,
anthem effects and pump-spell effects would behave identically, which violates CR 611.2c.

#### Multi-Layer Effects and Lock-In (CR 613.6)

Some effects span multiple layers simultaneously. For example, "becomes a 2/2 artifact creature"
affects Layer 4 (adds Artifact and Creature types) and Layer 7b (sets P/T to 2/2). Such effects
are stored as one effect record per layer component, all sharing the same `source_id` and
timestamp so that ordering between the components is consistent.

When a multi-layer effect uses a `Filter` target scope, CR 613.6 requires that the set of
affected permanents is locked when the effect first applies in its earliest layer, and that
same locked set is used for all later layers of the same effect — even if some affected permanents
no longer satisfy the filter condition by the time a later layer is evaluated.

Concretely: "all noncreature artifacts become 2/2 artifact creatures" matches noncreature
artifacts in Layer 4. By the time Layer 7b is evaluated, those permanents have already become
creatures and no longer match "noncreature artifact". Without CR 613.6 lock-in, they would not
receive the 2/2 from 7b. With lock-in, the permanent IDs captured at Layer 4 are reused for
Layer 7b.

Implementation requirement: multi-layer effect record groups must record a `locked_target_set`
(the IDs of permanents determined when the effect first applied in its earliest layer). This
locked set is used for all subsequent layer components of the same multi-layer effect.

Single-layer effects do not require lock-in — `Filter` effects evaluate fresh at their single
applicable layer.

### Timestamp Model (CR 613.7)

Timestamps determine the order in which effects apply within a layer when no dependency rule
overrides. The model must follow CR 613.7 exactly:

- **Permanents**: Each permanent receives a timestamp when it enters a zone (CR 613.7d). This
  timestamp is the permanent's own and is stored on the permanent's state record.
- **Static-ability effects**: A continuous effect generated by a permanent's static ability
  inherits the source permanent's timestamp (CR 613.7a). It does NOT receive a separately
  incremented counter.
- **Spell-resolution effects**: A continuous effect generated at spell or ability resolution
  receives its own timestamp from an incremented game-level counter at the time of creation
  (CR 613.7b). This is independent of any permanent's timestamp.
- **Simultaneous entry**: When multiple permanents enter a zone simultaneously (e.g. at the
  start of the game), timestamps are assigned in APNAP order — Active Player, then Non-Active
  Player, following turn order among players (CR 613.7m).

The game-level counter used for spell-resolution effects is separate from the permanent-entry
timestamp counter. Both are monotonically increasing integers scoped to the game instance.

### Duration Types

| Duration | Meaning |
|----------|---------|
| `UntilEndOfTurn` | Expires during the Cleanup step of the current turn |
| `WhileSourceOnBattlefield` | Active as long as the source permanent (identified by `source_id`) remains on the battlefield — used both for auras/enchantments affecting other permanents AND for a permanent's own static ability effects (the source is the permanent itself) |

The existing `EffectDuration::UntilEndOfTurn` variant is retained. One new variant is added.

`WhileSourceOnBattlefield` effects are removed from the active set whenever their source
permanent leaves the battlefield. This happens as a consequence of zone transition handling, not
as a separate cleanup phase.

Note: a permanent's own always-on static ability (e.g. Glorious Anthem granting +1/+1 to all
your creatures) uses `WhileSourceOnBattlefield` with `source_id` set to the permanent's own ID.
There is no distinct `Permanent` duration variant — this collapses the two concepts into one.

### Dependency Rules (CR 613.8)

Within each layer, effects are applied in timestamp order: the effect created earliest is applied
first. This is the default rule.

**Dependency exception** (CR 613.8): Effect A depends on effect B if applying B in the same layer
would change what A does. When dependency exists, B is applied before A regardless of timestamps.

For this spec, the only dependency case to implement is:
- An effect that sets a permanent's type (Layer 4) to a type that causes another Layer 7 effect to
  apply or not apply. Example: an effect that says "if this is a creature, it gets +2/+2". The
  type-setting effect (Layer 4) logically precedes the P/T effect (Layer 7) because they are in
  different layers — this is already handled by layer ordering. True same-layer dependency is
  uncommon enough to defer to a future spec.

The implementation must apply effects in timestamp order within each layer (CDAs first in layers
4 and 5 where CDA subtypes and CDA colors apply). Full dependency resolution across effects in
the same layer is out of scope.

### Effect Collection and Storage

The game maintains a single ordered list of all active continuous effects across all permanents.
Each effect specifies which permanents it affects via its target scope (`LockedSet` or `Filter`).
When computing effective characteristics for a permanent, the system filters this list to effects
that include that permanent and then applies them layer by layer.

Alternatively, effects generated by a permanent's own static ability may be stored on the
permanent's state directly (as they are now), but must participate in the same layer ordering
when queried. Both storage strategies must produce identical query results — which to use is an
implementation decision.

### Queries That Change

The following queries must be updated to be layer-aware:

| Query | Current behavior | New behavior |
|-------|-----------------|--------------|
| `effective_power(permanent_id)` | base + counters + ContinuousEffect sum | Layers 7a → 7d; uses post-Layer-4 types |
| `effective_toughness(permanent_id)` | base + counters + ContinuousEffect sum | Layers 7a → 7d; uses post-Layer-4 types |
| `effective_types(permanent_id)` | Returns `CardDefinition.types` (static) | Subtype CDAs then Layer 4 applied |
| `effective_colors(permanent_id)` | Returns `CardDefinition.colors()` (static) | Color CDAs then Layer 5 applied |
| `effective_abilities(permanent_id)` | Returns `CardDefinition.static_abilities` (static) | Layer 6 applied (no CDA ordering applies — CDAs cannot define keywords per CR 604.3a), including keyword counters and `CantHaveAbility` overrides |

The existing `current_power()` and `current_toughness()` methods on `PermanentState` are the
current implementation. They must remain as internal helpers but the engine-level queries
(accessible to combat resolution, SBA checks, and UI export) must route through the
layer-aware functions.

### State-Based Action Integration

The SBA check for "creature with toughness 0 or less" (CR 704.5f) must use `effective_toughness`
so that Layer 7 effects are considered. Likewise, the "creature with lethal damage" check uses
`effective_toughness`. Both are already delegated through `PermanentState::has_lethal_damage()`,
which calls `current_toughness()`. The migration path is to replace that call with the
layer-aware equivalent.

---

## Acceptance Criteria

All criteria must be verifiable by unit tests in `echomancy-core` without Bevy.

### Layer 7 P/T Calculation

- [x] A creature with base 2/2, no effects: `effective_power` = 2, `effective_toughness` = 2.
- [x] A 2/2 creature with one `+3/+3` Layer 7c effect: `effective_power` = 5, `effective_toughness` = 5.
- [x] A 2/2 creature with a Layer 7b "becomes 1/1" effect and a Layer 7c `+3/+3` effect:
  `effective_power` = 4, `effective_toughness` = 4 (1+3). Correct regardless of which effect
  has the earlier timestamp.
- [x] Two Layer 7c effects (`+2/+2` and `+1/+1`): result is base + 3/+3 total. Timestamp ordering
  does not affect the cumulative sum.
- [x] A 2/2 with two `+1/+1` counters (Layer 7c): `effective_power` = 4, `effective_toughness` = 4.
- [x] A 2/2 with two `-1/-1` counters (Layer 7c): `effective_power` = 0, `effective_toughness` = 0.
- [x] A 2/2 with one `+1/+1` counter and one `-1/-1` counter (both in Layer 7c): `effective_power` = 2,
  `effective_toughness` = 2 (they sum to +0/+0 within 7c). Note: in practice SBA 704.5q removes
  such counter pairs before this query is reached; this test exercises correctness in isolation.
- [x] A Layer 7c `+3/+3` effect with `UntilEndOfTurn` duration is absent from calculation after
  end-of-turn cleanup.
- [x] A 1/3 creature with a Layer 7c `+0/+1` effect and a Layer 7d switch effect:
  `effective_power` = 4, `effective_toughness` = 1 (after 7c P/T = 1/4, then 7d switches).
- [x] A 2/4 creature with a Layer 7b "becomes 3/5" effect and a Layer 7d switch effect:
  `effective_power` = 5, `effective_toughness` = 3 (after 7b P/T = 3/5, then 7d switches).

### Layer 4 Type Queries

- [x] A Land permanent with no type effects: `effective_types` = `[Land]`.
- [x] A Land with a Layer 4 effect granting `Creature`: `effective_types` = `[Land, Creature]`.
- [x] A Creature with a Layer 4 effect removing `Creature`: `effective_types` does not include
  `Creature`.
- [x] A Changeling (subtype CDA — "is every creature type"): `effective_types` includes all
  creature subtypes as determined by the CDA, evaluated before any timestamp-ordered Layer 4
  effects. Confirms CDA subtypes apply first within Layer 4.

### Layer 5 Color Queries

- [x] A colorless artifact with no effects: `effective_colors` = `[]`.
- [x] A colorless artifact with a Layer 5 "becomes blue" effect: `effective_colors` = `[Blue]`.
- [x] A red creature with a Layer 5 "loses all colors" effect: `effective_colors` = `[]`.
- [x] A permanent with a color CDA (analogous to Transguild Courier — "is all colors"): its color
  CDA is evaluated first within Layer 5, and a subsequent timestamp-ordered "becomes blue" effect
  applied on top results in `effective_colors` = `[Blue]`. Confirms color CDAs are evaluated
  before timestamp-ordered effects within Layer 5.

### Layer 6 Ability Queries

- [x] A creature with Flying on its card definition: `effective_abilities` includes `Flying`.
- [x] A creature with Flying on its card definition and a Layer 6 "loses Flying" effect:
  `effective_abilities` does not include `Flying`.
- [x] A creature without Trample and a Layer 6 "gains Trample" effect: `effective_abilities`
  includes `Trample`.
- [x] A creature bearing a keyword counter granting Flying: `effective_abilities` includes `Flying`.
- [x] A creature with a "gains Hexproof" Layer 6 effect and a `CantHaveAbility(Hexproof)` Layer 6
  effect active simultaneously: `effective_abilities` does not include `Hexproof` (the
  `CantHaveAbility` overrides regardless of timestamps).

### Target Scope — LockedSet vs Filter

- [x] A spell-resolution `LockedSet` effect ("all creatures get +1/+1 until end of turn"): a
  creature that enters the battlefield after the effect is created does NOT receive the +1/+1
  bonus. The effect applies only to permanents in the locked set.
- [x] A static-ability `Filter` effect ("creatures you control get +1/+1", e.g. Glorious Anthem):
  a creature entering the battlefield after the anthem is already active DOES receive the +1/+1
  bonus. The filter is re-evaluated on each query.

### Multi-Layer Effect Lock-In (CR 613.6)

- [x] An effect "all noncreature artifacts become 2/2 artifact creatures" (multi-layer: Layer 4
  adds Creature type, Layer 7b sets P/T to 2/2): a permanent that matched "noncreature artifact"
  when the effect first applied (and is captured in `locked_target_set`) receives the 2/2 P/T
  in Layer 7b even though it is now a creature (and no longer "noncreature") by that point.

### Timestamp Ordering

- [x] Two Layer 7b "becomes X/Y" effects: the one with the later timestamp wins (later effect
  overrides the earlier set-value in 7b, since both set, and order matters — older applied first
  means newer is applied last and wins).
- [x] Two Layer 7c effects with different timestamps: both modifiers are summed (order of addition
  does not affect a sum, but must still apply in timestamp order as required by CR 613.7).
- [x] A static-ability-generated Layer 7c effect uses the source permanent's timestamp, not a
  separately incremented counter.
- [x] A spell-resolution effect uses the game-level counter timestamp assigned at the time of
  spell resolution.

### CDA Identification

- [x] An effect with `is_cda = true` in Layer 4 is applied before effects with `is_cda = false`
  in the same layer, regardless of timestamps.
- [x] An effect with `is_cda = true` in Layer 5 is applied before effects with `is_cda = false`
  in the same layer, regardless of timestamps.

### Duration Handling

- [x] A `WhileSourceOnBattlefield` effect is active while the source permanent is on the battlefield.
- [x] A `WhileSourceOnBattlefield` effect is absent from calculations after its source permanent
  leaves the battlefield.
- [x] A permanent's own static ability effect (`WhileSourceOnBattlefield` with `source_id` = its
  own permanent ID) is removed when that permanent leaves the battlefield.

### SBA Integration

- [x] `has_lethal_damage()` on a 2/2 that has been set to 1/1 by a Layer 7b effect and received
  2 damage returns `true` (1/1 with 2 damage is lethal even though base toughness is 2).
- [x] A creature reduced to 0 effective toughness by Layer 7b triggers the toughness-0 SBA check.

---

## Out of Scope

The following are intentionally deferred to future specs:

- **Layer 1 (Copy effects)**: Cloning, copying permanents. Requires a separate copy-object spec.
- **Layer 2 (Control-changing effects)**: "Gain control of target creature." Requires a control
  model beyond the current owner/controller setup.
- **Layer 3 (Text-changing effects)**: "Change all instances of one word to another." Rarely
  relevant for MVP cards.
- **Full dependency resolution (CR 613.8)**: Same-layer circular dependencies. Handled by
  timestamp order only.
- **CDAs beyond CardsInControllerHand**: Devotion-based P/T, creature-count-based P/T, etc.
- **Effects scoped to multiple permanents**: "All creatures get +1/+1." The data model must
  accommodate this (one effect record per affected permanent, or a broadcast effect) but the
  specific implementation shape is left to the engineer.
- **Layer interactions involving planeswalkers**: Loyalty abilities are not continuous effects.
- **Replacement effects (CR 614)**: How effects modify events rather than characteristics. Separate spec.
- **CR 613.6 lock-in for single-layer Filter effects**: Lock-in is only required for multi-layer
  effects. Single-layer `Filter` effects always re-evaluate their target set fresh.

---

## Implementation Tracking

All acceptance criteria marked `[x]` above have been implemented and verified by unit tests.

### Migration Completed (2026-04-05)

The final legacy call sites have been migrated from `current_power()` / `current_toughness()` to
the layer-system-aware `effective_power()` / `effective_toughness()`:

- **`fight()` in `game/mod.rs`**: Now uses `self.effective_power()` for both creatures. Layer
  pump effects are correctly reflected in fight damage.
- **`bolster()` in `game/mod.rs`**: Now uses `self.effective_toughness()` to select the target.
  Bolster correctly picks the creature with the least *effective* toughness.
- **`adapt()` in `game/mod.rs`**: Now uses `self.effective_power()` to validate creature-ness.
- **Skulk check in `services/combat_declarations.rs`**: Added `effective_power_of()` to
  `CombatValidationContext` trait (with a default fallback for test contexts). `Game` overrides
  with the full layer pipeline. Skulk now compares effective powers, not base powers.
- **Legacy dual-write removed from `stack_resolution.rs`**: `ModifyPowerToughness` no longer
  writes to the per-permanent `ContinuousEffect` list. Only `GlobalContinuousEffect` is used.
- **CLIPS pipeline tests updated** in `infrastructure/clips/card_rules.rs` to assert via
  `game.effective_power()` / `game.effective_toughness()` and `game.global_continuous_effects`
  instead of the legacy `PermanentState::current_power()` / `current_toughness()`.
