# R11: Replacement Effects Framework (CR 614-615)

## Overview

Replacement effects are a fundamental MTG rules mechanism that modify an event **before** it
occurs, replacing what would have happened with something different. They are identified by the
words "instead" or "skip" on a card's oracle text.

Unlike triggered abilities, replacement effects do not use the stack, cannot be responded to,
and apply instantaneously as part of the event they modify. Per CR 614.1, if a replacement
effect would apply to an event, the modified version of the event occurs instead of the original.
The original event never happens.

**Design goal**: Introduce a replacement effect registry and interception layer so that game
events — specifically damage being dealt, permanents being destroyed, and permanents entering the
battlefield — can be intercepted, modified, or suppressed before they take effect. This is the
foundational framework that R12 (Prevention Effects) and CK1 (Regenerate) will build upon.

**Relationship to other systems**:
- The Layer System (LS1) handles *continuous* modifications to characteristics. Replacement effects
  handle *event* modifications. These are distinct: LS1 answers "what is this permanent's
  toughness right now?"; R11 answers "when damage is about to be dealt, does something modify that
  damage event?".
- Triggered abilities (R10) fire *after* an event. Replacement effects fire *instead of* an event
  — they intercept the event before it resolves.
- State-Based Actions (C5) run after events settle. A replacement effect that prevents a creature
  from dying changes what the SBA loop sees when it next runs.

---

## User Stories

**As a player**, I want a creature with Regenerate to survive lethal damage by having its
"destroy" event replaced with "tap, remove damage, remove from combat", so that the creature
stays on the battlefield rather than going to the graveyard.

**As a player**, I want a damage prevention effect ("prevent the next N damage that would be
dealt to target creature") to intercept the damage event and reduce or eliminate the damage
before it is marked, so that my creature does not die to combat or burn.

**As a developer**, I want to register a replacement effect from CLIPS (when a spell resolves)
or from a permanent's static ability (always active while on the battlefield) using a single
uniform API, so that future card effects can participate in the replacement system without
changes to the interception logic.

**As a QA validator**, I want unit tests that confirm each interception point catches the right
events, applies the correct replacement, and respects the "apply once" rule so that a replacement
effect is never applied twice to the same event.

---

## Player Experience

Replacement effects are invisible in the sense that the player never explicitly "activates" one.
They apply automatically. The player's experience is:

1. An action occurs (e.g. a creature is dealt lethal damage in combat).
2. The engine checks whether any replacement effects apply to the resulting event (e.g. "if this
   creature would be destroyed, regenerate it instead").
3. If a replacement applies, the modified outcome occurs and is reflected immediately in the UI
   (e.g. the creature remains on the battlefield, tapped, rather than moving to the graveyard).
4. The player sees the final result — they do not see the original unmodified event.

When multiple replacement effects could apply to the same event (CR 614.5), the affected player
is asked to choose which applies first. For the MVP, this choice is resolved by timestamp order
(earliest-created effect applies first) without a player prompt. The player-choice flow is
deferred to a future spec.

---

## Game Rules and Mechanics

### What a Replacement Effect Is

A replacement effect watches for a specific class of event and, when that event would occur,
replaces it with a different outcome. Key properties (CR 614.1, 614.4, 614.6):

- **Pre-event**: The replacement happens before the original event. The original event does not
  occur. The modified event occurs in its place.
- **No stack**: Replacement effects do not go on the stack. Players cannot respond to a
  replacement effect being applied (they can respond to the spell or ability that created the
  replacement effect, before it resolves — but not to the replacement itself).
- **Must pre-exist**: A replacement effect must exist before the event it modifies (CR 614.4).
  An effect that would come into existence as a result of the event cannot modify that event.
- **Apply once**: Once a replacement effect has been applied to an event, it cannot apply again
  to that same event (CR 614.6). The modified event may still be subject to other replacement
  effects, but not to the one already applied.

### The Three MVP Event Categories

This spec covers three event categories that serve as interception points:

#### Category A: Damage Events

A damage event captures: the source of the damage, the target (a player or a permanent), and the
amount. Replacement effects that intercept damage events include:

- **Prevention**: "Prevent the next N damage that would be dealt to [target]." The amount is
  reduced by up to N. If reduced to zero, no damage is marked or dealt. The prevention shield
  decrements by the amount prevented.
- **Redirection**: "If damage would be dealt to [target], it deals that damage to [other target]
  instead." The target of the damage event changes.
- **Substitution**: "If [permanent] would be dealt damage, [something else] happens instead."

The damage interception point is at the moment damage is *about to be marked* — before damage
counters are placed on a permanent or life is subtracted from a player. This is the point at
which the engine must check for applicable damage replacement effects.

#### Category B: Destroy Events

A destroy event captures: the permanent being destroyed and the reason (lethal damage, an
explicit "destroy" effect, etc.). Replacement effects that intercept destroy events include:

- **Regeneration**: "If [permanent] would be destroyed, instead tap it, remove all damage from
  it, and remove it from combat." The permanent is not moved to the graveyard.
- **Exile instead**: "If [permanent] would be destroyed, exile it instead." The permanent goes
  to exile rather than the graveyard.

The destroy interception point is immediately before a permanent is moved to the graveyard as a
result of an explicit "destroy" effect (e.g. "Destroy target creature") or the lethal-damage SBA
(CR 704.5g — creature has damage marked on it greater than or equal to its toughness). It is NOT
triggered by the zero-toughness SBA (CR 704.5f — creature has 0 or less toughness). Per CR 704.5f,
zero-toughness is treated as "put into the graveyard" rather than "destroy", so regeneration
shields and other destroy-replacement effects do not intercept it. It is also NOT triggered by
zone changes caused by sacrifice, exile effects, or bounce effects — those are not "destroy"
events.

The `DestroyReason` carried by each destroy event must distinguish at minimum: `LethalDamage`,
`DestroyEffect`, and `ZeroToughness`. The interception logic must only activate replacement
effects for `LethalDamage` and `DestroyEffect` reasons; a `ZeroToughness` event bypasses the
replacement framework entirely and proceeds directly to moving the permanent to the graveyard.

#### Category C: ETB Replacement Events

An ETB replacement event intercepts a permanent entering the battlefield and modifies the
conditions under which it enters. Examples:

- "CARDNAME enters the battlefield tapped." — already modeled as `StaticAbility::EntersTapped`
  in the existing domain. This spec formalizes that existing behavior as an ETB replacement
  effect so it participates in the replacement framework (allowing future effects to interact
  with it correctly).
- "CARDNAME enters the battlefield with N +1/+1 counters." — partially implemented; this spec
  formalizes it as an ETB replacement.

ETB replacement effects differ from the above two categories: they do not replace "entering the
battlefield" with "not entering" — the permanent still enters. They modify the *state* the
permanent enters with (tapped vs. untapped, with counters vs. without).

### Replacement Effect Registration

A replacement effect is registered from one of two sources:

**Source 1 — Static ability of a permanent**: The effect is always active while the permanent
is on the battlefield. It is created when the permanent enters the battlefield and removed when
the permanent leaves. Examples: a permanent with Regenerate written in its rules text; a global
effect enchantment that says "whenever a creature would be destroyed, exile it instead".

**Source 2 — Spell or activated ability resolution**: The effect is created at resolution time
and has a limited duration (typically "until end of turn" or "the next time this would happen").
Examples: a regeneration ability activated in response to a destroy effect; a prevention shield
("prevent the next 3 damage to target creature") from a resolved instant.

Each registered replacement effect carries:

| Field | Description |
|-------|-------------|
| `effect_id` | Unique identifier for this replacement effect instance |
| `source_id` | ID of the source permanent or spell that created this effect |
| `controller_id` | Player who controls the source |
| `event_filter` | Which event category and target this effect watches for |
| `replacement` | What happens instead (the replacement outcome) |
| `duration` | When this effect expires |
| `timestamp` | Monotonically increasing integer from the game-level counter |
| `applied_to` | Set of event instance IDs this effect has already been applied to (starts empty; enforces CR 614.6) |

### Event Interception Protocol

When an event in one of the three MVP categories is about to occur, the engine executes this
protocol:

1. **Collect candidates**: Find all active replacement effects whose `event_filter` matches the
   event (correct category, correct target).
2. **Remove already-applied**: Exclude any effect whose `applied_to` set contains this event's
   instance ID (CR 614.6 — apply-once rule).
3. **If no candidates**: Proceed with the original event unmodified.
4. **If one candidate**: Apply it. Mark the event ID in the effect's `applied_to` set. If the
   effect has a "use once" duration (e.g. a prevention shield that depletes), update its
   remaining count. If the remaining count reaches zero, remove the effect from the registry.
5. **If multiple candidates**: Apply in timestamp order (oldest first) for the MVP. After
   applying the first, re-evaluate candidates against the *modified* event. Continue until no
   candidates remain or the event results in "nothing happens".

An event instance ID is a monotonically increasing counter assigned immediately before the
interception check begins. This is distinct from the timestamp on the replacement effect.

### Ordering When Multiple Replacements Apply (CR 614.5)

CR 614.5 states that when multiple replacement effects could apply to the same event, the
affected player (or controller of the affected permanent) chooses the order. The MVP simplifies
this to timestamp order (oldest replacement effect applies first). This produces correct results
in most practical cases and avoids the need for a player-choice UI prompt in Phase 1.

The primary case where timestamp ordering diverges from correct CR 614.5 behavior is when two
different players each control a replacement effect applicable to the same event. In that scenario
the rules grant the choice to the affected player (not the controller of either replacement
effect), which can produce a different ordering than timestamp order. The MVP accepts this
divergence; the player-choice prompt for cross-controller ordering is deferred.

The full player-choice UI for ordering multiple replacements is out of scope for this spec.

### The "Apply Once" Rule (CR 614.6)

After a replacement effect is applied to an event, the modified event may trigger further
replacement checks. However, the already-applied effect cannot apply again. This prevents
infinite loops (e.g. two "instead exile it" effects do not cycle forever).

The `applied_to` set on each replacement effect record tracks which event instances it has
already modified. Before applying an effect to an event, the engine checks this set. If the
event instance ID is already present, the effect is skipped.

### Duration and Lifecycle of Replacement Effects

| Duration variant | Meaning |
|-----------------|---------|
| `UntilEndOfTurn` | Effect is removed during the Cleanup step |
| `WhileSourceOnBattlefield` | Effect is removed when its source permanent leaves the battlefield |
| `NextOccurrence` | Effect is removed after it applies once (e.g. single-use regeneration shields) |
| `UntilDepleted { remaining: N }` | Effect tracks a remaining amount; decremented on each use and removed when it reaches zero (e.g. prevention shields with a fixed damage budget) |

The `NextOccurrence` duration is appropriate for regeneration shields, which are either consumed
or not. Prevention shields with a fixed damage budget (e.g. "prevent the next 3 damage") use
`UntilDepleted` instead: each application reduces `remaining` by the amount prevented, and the
effect is removed only when `remaining` reaches zero. If a 3-point shield prevents 2 damage, it
remains in the registry with `remaining = 1` and is not removed. If it prevents 3 or more, it is
removed. This distinction is mandatory because removing a partially-depleted shield after its
first use would be incorrect rules behavior.

### Regenerate as a Replacement Effect

Regenerate (CR 701.15) is the canonical example of a destroy replacement effect. When a creature
with an active regeneration shield would be destroyed:

1. The destroy event is intercepted.
2. The regeneration shield is consumed (the `NextOccurrence` effect is removed).
3. Instead of moving to the graveyard, the creature: (a) has all damage removed, (b) is tapped,
   (c) is removed from combat if it is currently attacking or blocking.
4. The creature remains on the battlefield.

The regeneration shield is created by an activated ability (e.g. "{1}{G}: Regenerate this
creature"). That activated ability resolves, creates a `NextOccurrence` replacement effect, and
the shield sits in the registry until a destroy event triggers it or the turn ends.

Note that regeneration only intercepts destroy events caused by `LethalDamage` or `DestroyEffect`.
A creature killed by the zero-toughness SBA (CR 704.5f) is not destroyed — it is placed directly
into the graveyard — so its regeneration shield is not consumed and will not save it.

Note: A creature with Indestructible does not benefit from a regeneration shield because
Indestructible prevents destruction before the event reaches the replacement framework — but the
SBA for "creature with 0 or less toughness" still kills it regardless of regeneration or
Indestructible. The interaction between Indestructible and regeneration is out of scope here, as
Indestructible is already handled correctly in SBA.

### ETB Replacement Formalization

The existing `StaticAbility::EntersTapped` and the partial +1/+1 counter-on-ETB implementation
should be migrated to participate in the replacement framework without changing their external
behavior. This formalization allows future ETB replacements (e.g. "if a creature would enter
the battlefield, it enters with an additional +1/+1 counter") to layer correctly with existing
ones.

Note: "enters the battlefield tapped" and "enters the battlefield with N +1/+1 counters" are
technically self-replacement effects under CR 614.15-617 — they are replacement effects that
apply to an event generated by the permanent itself as it enters. These two specific cases are
included in the MVP scope because they are already partially implemented and are low-complexity
self-replacements with no ambiguity about applicability. Full self-replacement rules (covering
the edge cases in CR 614.15-617, such as effects that replace entering with a copy of the
permanent itself) remain deferred.

The existing `enter_battlefield()` function in `zone_transitions.rs` already checks
`has_static_ability(EntersTapped)` and applies it directly. This spec does not require refactoring
that call site. The formalization is a conceptual alignment: new ETB replacements introduced by
future card rules must go through the replacement registry rather than ad-hoc checks in
`enter_battlefield()`.

### Interaction with State-Based Actions

The SBA loop (C5) checks for creatures to destroy after every game action. When a destroy event
fires from the SBA loop (e.g. creature has toughness 0 or lethal damage), the replacement
framework intercepts it before the creature is moved to the graveyard. The SBA loop is unaware
of the replacement layer — it fires a logical "destroy this permanent" event and the
interception happens transparently within that event processing.

This means: if a regeneration shield is active and the SBA fires a destroy event for a creature
with lethal damage, the replacement effect applies, the shield is consumed, damage is cleared,
and the creature is tapped. The SBA loop then re-runs and finds the creature is no longer at
lethal damage — it stays on the battlefield.

### Interaction with Combat Damage

The combat damage step produces `DamageAssignment` records (already in
`services/combat_resolution.rs`). Each assignment corresponds to a damage event. The replacement
framework must intercept these events before the damage is marked. For the MVP, this means the
`apply_combat_damage` call site (in `game/combat_damage.rs`) must run each assignment through
the interception protocol before marking damage on permanents or subtracting life from players.

---

## Acceptance Criteria

All criteria must be verifiable by unit tests in `echomancy-core` without Bevy.

### Framework Registration

- [x] A replacement effect can be registered in the game's replacement registry with a source ID,
  event filter, replacement outcome, duration, and timestamp.
- [x] A replacement effect with `WhileSourceOnBattlefield` duration is removed from the registry
  when its source permanent leaves the battlefield.
- [x] A replacement effect with `UntilEndOfTurn` duration is removed from the registry during
  the Cleanup step.
- [x] A replacement effect with `NextOccurrence` duration is removed from the registry
  immediately after it applies once.

### Apply-Once Rule (CR 614.6)

- [x] When a replacement effect is applied to an event, its `applied_to` set is updated with
  the event instance ID.
- [x] A replacement effect whose `applied_to` set already contains the current event instance
  ID is not applied again to that event, even if it would otherwise match the event filter.
- [x] Two different replacement effects can both be applied to the same event in sequence
  (the apply-once rule only prevents a *single* effect from applying twice, not two
  distinct effects from applying to the same base event).

### Damage Replacement — Prevention Shield

- [x] A prevention shield effect (e.g. "prevent the next 3 damage to creature X") intercepts
  a damage event targeting creature X and reduces the damage by up to 3.
- [x] If a 3-point prevention shield intercepts a 5-damage event, the result is 2 damage
  marked, and the shield is consumed (removed from the registry).
- [x] If a 3-point prevention shield intercepts a 2-damage event, the result is 0 damage
  marked, and the shield decrements to 1 remaining (still active in the registry).
- [x] If a 3-point prevention shield intercepts a 3-damage event, the result is 0 damage
  marked, and the shield is fully consumed (removed from the registry).
- [x] A prevention shield targeting creature X does not intercept a damage event targeting
  creature Y or a player.
- [x] A creature with 0 damage marked after a prevention shield does not trigger the lethal
  damage SBA.

### Destroy Replacement — Regeneration Shield

- [x] A regeneration shield registered for creature X intercepts a destroy event for X.
- [x] After interception: creature X is tapped, has all damage removed, and remains on the
  battlefield (not moved to the graveyard).
- [x] After interception: the regeneration shield is consumed (removed from the registry).
- [x] A regeneration shield registered for creature X does not intercept a destroy event for
  creature Y.
- [x] After a regeneration shield fires during combat, the creature is no longer marked as
  attacking or blocking (it is removed from combat as part of the regeneration replacement).
- [x] A creature with an active regeneration shield that receives lethal damage in combat,
  then has the destroy event intercepted by the shield, remains on the battlefield after the
  SBA loop re-runs.
- [x] A creature destroyed by "exile it" (not a destroy event) does NOT trigger the
  regeneration shield — regeneration only applies to destroy events, not zone changes caused
  by exile effects.

### Multiple Replacement Effects

- [x] When two replacement effects match the same event, both with `NextOccurrence` duration,
  the one with the earlier timestamp (lower value) applies first.
- [x] After the first replacement effect applies, the modified event is re-evaluated against
  remaining candidates (excluding the already-applied one).
- [x] If applying the first replacement effect results in "no event" (e.g. damage reduced to 0),
  the second replacement effect is not applied (there is no remaining event to replace).

### ETB Replacement

- [x] A permanent with `StaticAbility::EntersTapped` enters the battlefield in the tapped state
  (existing behavior is preserved through the formalization).
- [x] A replacement effect that says "permanent X enters the battlefield with N +1/+1 counters"
  causes X to have those counters immediately upon entering the battlefield, visible before
  any ETB triggered abilities are evaluated.

### Integration with SBA Loop

- [x] A creature at lethal damage that has an active regeneration shield: when the SBA fires a
  destroy event, the shield intercepts it, and the SBA loop re-runs and finds the creature
  alive (damage cleared) — the creature is not moved to the graveyard.
- [x] A creature at lethal damage with no active replacement effects: the SBA destroy event
  proceeds unmodified, and the creature is moved to the graveyard.

### Integration with Combat Damage

- [x] A prevention shield on a blocking creature intercepts the combat damage event from an
  attacker before the damage is marked.
- [x] A player with a prevention shield (prevent the next N damage to you) intercepts
  unblocked combat damage before life is subtracted.

---

## Out of Scope

The following are intentionally deferred:

- **Draw replacement effects** (Dredge, Notion Thief): Requires a `DrawEvent` interception
  point and CLIPS rules for draw-replacement. Future spec.
- **Zone-change replacement effects** (Rest in Peace — "if a card would go to the graveyard,
  exile it instead"): Requires intercepting all zone change events, not just destroy events.
  Future spec.
- **"Skip" effects** (e.g. "skip your draw step"): Replacement effects without an "instead"
  clause per CR 614.1a. Future spec alongside TR1/TR2 step trigger work.
- **Self-replacement limitation** (CR 614.15-617): The edge cases of CR 614.15-617 are deferred.
  The simple self-replacement cases "enters tapped" and "enters with N counters" are in scope for
  the MVP (see ETB Replacement Formalization above); all other self-replacement scenarios are not.
- **Player-choice ordering of multiple replacements** (CR 614.5 full compliance): The MVP uses
  timestamp order. The UI for prompting the affected player to choose which replacement applies
  first is deferred.
- **Replacement effects on "if damage would be dealt by [source]" style filters**: Only
  target-based filters (by permanent ID or player ID) are in scope for MVP.
- **Copy effects that interact with replacement** (CP1): Copy effects may interact with
  replacement effects in Layer 1. Deferred to CP1.
- **Protection from X** (K6): Protection prevents damage, enchanting, blocking, and targeting
  ("DEBT") and is partly a replacement-effect-like mechanism. Deferred to K6.
- **The existing `StaticAbility::EntersTapped` code path refactor**: The existing direct check
  in `enter_battlefield()` is preserved as-is. Only *new* ETB replacement effects from CLIPS
  rules go through the replacement registry. Refactoring the existing path is out of scope.
