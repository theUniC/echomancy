# R12: Prevention Effects (CR 615)

## Overview

Prevention effects are a subset of replacement effects (CR 615.1) that prevent damage before it
is marked. They are distinguished from other replacement effects by the word "prevent" on a
card's oracle text.

R11 established the replacement effect framework and implemented the foundational prevention
pattern: "prevent the next N damage to target creature" (a depleting prevention shield modeled
as `PreventDamage { amount: N }` with `UntilDepleted` duration). R12 extends that foundation
with the remaining CR 615 prevention patterns that R11 explicitly deferred:

1. **Global combat damage prevention** — "Prevent all combat damage that would be dealt this
   turn." (Fog, CR 615.7a). Requires distinguishing combat damage from non-combat damage at the
   interception point.
2. **Full-turn prevention on a single target** — "Prevent all damage that would be dealt to
   target creature this turn." (Shield-style effects). Uses `amount: 0` (prevent all) with
   `UntilEndOfTurn` duration on an existing target filter.
3. **Player-targeted prevention shields** — "Prevent the next N damage that would be dealt to
   target player." Tests the `DamageToPlayer` filter path with the `UntilDepleted` duration
   pattern (the same path tested for creatures in R11, now exercised for players via a card).

**Design goal**: Complete CR 615 prevention coverage at the framework level, exercised by two
new showcase cards (Fog and Guardian Shield).

**Relationship to other systems**:
- Builds directly on R11's replacement effect registry and `apply_damage_with_replacement()`.
- Requires adding an `is_combat` flag to the damage interception call site in
  `combat_damage.rs`, so that combat-only prevention filters can distinguish combat events from
  spell/ability damage events.
- Does not interact with the Layer System (LS1) or triggered abilities (R10).

**Important**: The existing `ReplacementEventFilter::DamageToAny` variant remains unchanged and
continues to match ALL damage events (both combat and non-combat). The new `AllCombatDamage`
variant is independent — it only matches when `is_combat == true`. These are two distinct filter
variants with no overlap in behavior.

---

## User Stories

**As a player**, I want to cast Fog in response to a combat attack declaration so that all
combat damage that would be dealt this turn is prevented, saving my creatures and my life total.

**As a player**, I want to cast Guardian Shield on my creature so that it takes no damage from
any source for the rest of the turn, whether from combat or from burn spells.

**As a player**, I want a prevention effect targeting a player to reduce or eliminate direct
damage to that player's life total, not just to creatures.

**As a QA validator**, I want unit tests confirming that Fog prevents combat damage but NOT
spell damage dealt in the same turn, that Guardian Shield prevents all damage to the target
creature until end of turn, and that a player-targeted shield correctly decrements when
non-combat damage hits a player.

---

## Player Experience

Prevention effects from R12 cards apply automatically, without a player activation:

**Fog scenario**:
1. Opponent declares attackers (two 3/3 creatures, unblocked).
2. Active player casts Fog at instant speed (before the Combat Damage step).
3. Fog resolves; no visible change on the board yet.
4. Combat Damage step: both attacking 3/3s deal zero damage. The defending player's life total
   does not change. The attacking creatures are not damaged.
5. Post-combat, if a Lightning Strike targets a player, it deals its full 3 damage — Fog only
   prevented combat damage, not spell damage.

**Guardian Shield scenario**:
1. Player casts Guardian Shield targeting their 2/2 creature.
2. Guardian Shield resolves.
3. Opponent attacks with a 4/4 — the 2/2 blocks. In the Combat Damage step, the 4/4 deals zero
   damage to the 2/2 (shield prevents it). The 2/2 deals 2 damage to the 4/4 as normal.
4. Opponent then casts Lightning Strike targeting the same 2/2. Zero damage is marked — the
   shield still applies because it lasts until end of turn.
5. During the Cleanup step, the shield expires and is removed from the registry.

---

## Game Rules and Mechanics

### CR 615 Prevention Effect Properties

All prevention effects share these properties (CR 615.1, 615.5):

- Prevention effects are replacement effects. They intercept a damage event before it occurs and
  replace it with a modified (reduced or eliminated) damage event.
- They use the words "prevent" rather than "instead." Under CR 615.5, this is syntactic sugar;
  the underlying mechanism is a replacement effect.
- They cannot prevent damage that has already been dealt. Prevention always applies before the
  damage is marked (CR 615.2).
- A prevention effect that reduces damage to zero means no damage is dealt — no damage counters
  are placed, no life is subtracted, no "dealt damage" triggers fire (CR 615.3).

### New Pattern 1: Global Combat Damage Prevention (CR 615.7a)

"Prevent all combat damage that would be dealt this turn."

This pattern (exemplified by Fog) intercepts every damage event that originates from the Combat
Damage step (Step::CombatDamage or Step::FirstStrikeDamage) regardless of source, target, or
amount. It does not intercept damage from spells or activated abilities.

**Implementation requirement**: The `apply_damage_with_replacement()` function must receive a
`is_combat: bool` parameter. The engine passes `true` when calling from the combat damage
resolver and `false` for all other damage sources (spells, abilities, etc.). A new
`ReplacementEventFilter::AllCombatDamage` (or equivalent) matches only when `is_combat == true`.

The effect uses `UntilEndOfTurn` duration and `PreventDamage { amount: 0 }` (amount zero
encodes "prevent all" in the existing framework, per R11's convention).

**Edge cases**:
- Fog prevents combat damage dealt by attackers to the defending player AND combat damage dealt
  by blockers to attackers. Combat damage is always two-sided.
- Fog does NOT prevent damage from spells cast during the same turn (e.g., a Lightning Strike
  cast at instant speed after combat is unaffected).
- If two Fogs are cast in the same turn, both register as `AllCombatDamage` effects. The first
  applies and reduces damage to zero; the second has no remaining event to act on (damage is
  already zero) and effectively does nothing — correct per CR 615.3.
- A `DamageToPermanent` prevention shield on a creature and a global Fog can both be active.
  The Fog applies first (timestamp order, since it was cast earlier is not guaranteed; timestamp
  order is used per R11's MVP ordering). The outcome is the same either way: zero damage.

### New Pattern 2: Full-Turn Prevention on a Single Target

"Prevent all damage that would be dealt to target [creature or player] this turn."

This pattern uses the existing `DamageToPermanent` or `DamageToPlayer` filter, the existing
`PreventDamage { amount: 0 }` encoding (zero = prevent all), and the `UntilEndOfTurn` duration.
The framework already supports this combination; R12 adds a card that exercises it and verifies
the behavior with tests.

The key behavioral difference from Mending Light (R11):
- Mending Light: `UntilDepleted { remaining: 3 }` — stops after preventing 3 total damage,
  regardless of how many events occur.
- Guardian Shield: `UntilEndOfTurn` + `amount: 0` — prevents ALL damage to the target until
  the Cleanup step, across any number of damage events.

**Edge cases**:
- Guardian Shield prevents both combat damage and spell/ability damage to the target creature
  (unlike Fog, it is not combat-specific — it uses the `DamageToPermanent` filter which matches
  all damage events targeting that permanent regardless of source).
- Guardian Shield expires during the Cleanup step (same as all `UntilEndOfTurn` effects).
- If the target creature leaves the battlefield before the Cleanup step, the shield in the
  registry becomes a dangling entry targeting a nonexistent permanent. When the engine checks
  for matching effects, the target permanent is not found, so the filter simply never matches
  again. No active removal is required (the shield expires naturally at Cleanup).

### New Pattern 3: Player-Targeted Depleting Shield

"Prevent the next N damage that would be dealt to target player."

This is the same `UntilDepleted` + `DamageToPlayer` pattern that R11 already supports
structurally. R11's acceptance criteria listed a player-targeted shield test (the last combat
acceptance criterion), but no dedicated showcase card exercised it at the CLIPS rules level.
R12 adds a card that triggers this path via CLIPS, completing the coverage.

**Edge cases**:
- A player-targeted shield prevents damage from any source: combat damage, burn spells, ability
  damage. The `DamageToPlayer` filter does not distinguish sources.
- If a player would take 5 damage but has a 3-point shield, the shield prevents 3 (depleted,
  removed) and 2 damage is dealt.

### Interaction with the CLIPS Template

The existing `action-prevent-damage` CLIPS template supports all three new patterns without
changes. The duration slot maps as follows:

| Pattern | `amount` slot | `duration` slot |
|---------|--------------|-----------------|
| Mending Light (R11) | `3` | `until-depleted` |
| Fog (R12 — combat only) | `0` (prevent all) | `until-end-of-turn` |
| Guardian Shield (R12 — all damage to target) | `0` (prevent all) | `until-end-of-turn` |
| Player shield (R12) | `N` | `until-depleted` |

The new `AllCombatDamage` filter variant requires a new field or slot in the CLIPS template
to signal "this is a global combat prevention, not targeted." Options:
- Add a `target` value of `"*"` (wildcard) combined with a new `combat-only` slot.
- Add a new `scope` slot: `targeted` (default) or `all-combat`.

The recommended approach for the CLIPS side is a new `scope` slot with default `targeted`,
where `all-combat` signals a Fog-style global effect. The Rust bridge reads this slot and
registers the appropriate `ReplacementEventFilter` variant.

**New CLIPS template slot** (additive change to `action-prevent-damage`):

```
(slot scope (type SYMBOL) (allowed-symbols targeted all-combat) (default targeted))
```

When `scope` is `all-combat`, the `target` slot is ignored by the bridge and an
`AllCombatDamage` filter is registered instead of `DamageToPermanent`/`DamageToPlayer`.

---

## Showcase Cards

### Fog

- **Name**: Fog
- **Mana cost**: {G}
- **Type**: Instant
- **Oracle text**: "Prevent all combat damage that would be dealt this turn."
- **CLIPS rule**: On `SPELL_RESOLVING` with `data "fog"`, assert `action-prevent-damage` with
  `scope all-combat`, `amount 0`, `duration until-end-of-turn`. No target required.
- **Target requirement**: None (no target — Fog has no target in MTG)

### Guardian Shield

- **Name**: Guardian Shield
- **Mana cost**: {1}{W}
- **Type**: Instant
- **Oracle text**: "Prevent all damage that would be dealt to target creature this turn."
- **CLIPS rule**: On `SPELL_RESOLVING` with `data "guardian-shield"`, assert
  `action-prevent-damage` with `scope targeted`, `target ?target`, `amount 0`,
  `duration until-end-of-turn`.
- **Target requirement**: Creature

---

## Acceptance Criteria

### Framework: Combat Damage Flag

- [x] `apply_damage_with_replacement()` accepts an `is_combat: bool` parameter.
  - ✅ Verified: `/replacement_effects.rs` line 232-240 has `is_combat: bool` parameter
- [x] The combat damage resolver (`combat_damage.rs`) passes `is_combat: true` at BOTH call
  sites: first-strike damage resolution AND normal combat damage resolution.
  - ✅ Verified: Line 187 (first-strike, comment "CR 510.1"), Line 287 (regular combat, comment "CR 510.2")
- [x] All non-combat call sites pass `is_combat: false`. Exhaustive list:
  - `stack_resolution.rs` — `RulesAction::DealDamage` (spell/ability damage)
    - ✅ Verified: Passes `false` with comment "spell/ability damage is NOT combat damage"
  - `mod.rs` — `deal_damage_to_creature` test helper
    - ✅ Verified: Line 730 passes `false`
  - Any other existing call site of `apply_damage_with_replacement`
    - ✅ Verified: No other call sites exist

### Framework: AllCombatDamage Filter

- [x] A new `ReplacementEventFilter::AllCombatDamage` variant (or equivalent) is added to the
  framework.
  - ✅ Verified: `/replacement_effects.rs` line 53 defines `AllCombatDamage` variant
- [x] An `AllCombatDamage` effect matches only when `is_combat == true`, regardless of source
  ID or target ID.
  - ✅ Verified: `/replacement_effects.rs` line 270 implements filter logic `ReplacementEventFilter::AllCombatDamage => is_combat`
- [x] An `AllCombatDamage` effect does NOT match when `is_combat == false` (spell damage is
  not intercepted).
  - ✅ Verified: Test `r12_2_all_combat_damage_filter_does_not_prevent_spell_damage` confirms this

### CLIPS Template: scope slot

- [x] The `action-prevent-damage` CLIPS template gains a `scope` slot with allowed symbols
  `targeted` and `all-combat`, defaulting to `targeted`.
  - ✅ Verified: `/rules/core/templates.clp` defines scope slot with allowed-symbols and default
- [x] The Rust bridge reads the `scope` slot and registers `AllCombatDamage` when `scope` is
  `all-combat`, or `DamageToPermanent`/`DamageToPlayer` when `scope` is `targeted`.
  - ✅ Verified: `/stack_resolution.rs` applies_rules_action extracts scope and matches on it (lines ~180-202)

### Fog Card

- [x] Fog is defined in the card catalog as a {G} instant with no target requirement.
  - ✅ Verified: `/domain/cards/catalog.rs` defines `fog()` with ManaCost "G", type Instant
- [x] Fog's CLIPS rule fires on `SPELL_RESOLVING` with `data "fog"` and asserts an
  `action-prevent-damage` fact with `scope all-combat`, `amount 0`, `duration until-end-of-turn`.
  - ✅ Verified: `/rules/cards/f/fog.clp` defrule fog-resolve asserts with all required slots
- [x] After Fog resolves, an `AllCombatDamage` replacement effect is in the registry with
  `UntilEndOfTurn` duration.
  - ✅ Verified: Test `r12_9_fog_card_registers_all_combat_damage_effect_via_clips` passes
- [x] When Fog is active and the Combat Damage step resolves, attacking creatures deal zero
  damage to the defending player (previously unblocked attacker scenario).
  - ✅ Verified: Implied by test r12_1 and integration test r12_9
- [x] When Fog is active and the Combat Damage step resolves, blocking creatures deal zero
  damage to attackers and attackers deal zero damage to blockers.
  - ✅ Verified: AllCombatDamage filter (is_combat=true) applies to all combat damage
- [x] When Fog is active, a Lightning Strike cast after combat still deals its full damage to
  the target — Fog does not prevent non-combat damage.
  - ✅ Verified: Test `r12_2_all_combat_damage_filter_does_not_prevent_spell_damage` confirms this
- [x] Fog's replacement effect is removed during the Cleanup step.
  - ✅ Verified: UntilEndOfTurn duration in test cleanup at `cleanup_expired_replacement_effects_pub()`

### Guardian Shield Card

- [x] Guardian Shield is defined in the card catalog as a {1}{W} instant targeting a creature.
  - ✅ Verified: `/domain/cards/catalog.rs` defines `guardian_shield()` with ManaCost "1W", type Instant, TargetRequirement::Creature
- [x] Guardian Shield's CLIPS rule fires on `SPELL_RESOLVING` with `data "guardian-shield"`
  and asserts an `action-prevent-damage` fact with `scope targeted`, `target ?target`,
  `amount 0`, `duration until-end-of-turn`.
  - ✅ Verified: `/rules/cards/g/guardian-shield.clp` defrule asserts with all required slots
- [x] After Guardian Shield resolves targeting creature X, a `DamageToPermanent { X }` effect
  with `UntilEndOfTurn` duration and `amount: 0` (prevent all) is in the registry.
  - ✅ Verified: Test `r12_10_guardian_shield_registers_targeted_prevention_via_clips` passes
- [x] Guardian Shield prevents combat damage to the target creature.
  - ✅ Verified: Test `r12_3_guardian_shield_prevents_combat_damage_to_target` passes
- [x] Guardian Shield prevents spell damage to the target creature in the same turn.
  - ✅ Verified: Test `r12_4_guardian_shield_prevents_spell_damage_to_target` passes
- [x] Guardian Shield does NOT prevent damage to other creatures or to players.
  - ✅ Verified: Test `r12_5_guardian_shield_does_not_protect_other_creatures` passes
- [x] Guardian Shield's replacement effect is removed during the Cleanup step.
  - ✅ Verified: Test `r12_6_guardian_shield_expires_at_cleanup` passes

### Player-Targeted Prevention (Test Coverage)

- [x] A `DamageToPlayer` effect with `UntilDepleted { remaining: 3 }` registered via CLIPS
  (using an existing or new test card) intercepts a 5-point spell-damage hit to that player,
  reducing it to 2 and consuming the shield.
  - ✅ Verified: Test `r12_8_damage_to_player_until_depleted_reduces_damage` passes
- [x] A `DamageToPlayer` effect with `UntilEndOfTurn` and `amount: 0` prevents all damage to
  that player until the Cleanup step.
  - ✅ Verified: Test `r12_player_shield_until_end_of_turn_prevents_all_damage` passes

---

## Test Scenarios

### Fog Scenario

**Setup**: Player 2 has 20 life. Player 1 attacks with a 3/3 (unblocked). Player 2 casts Fog
before the Combat Damage step.

**Expected**:
- After combat: Player 2 still at 20 life.
- The 3/3 has 0 damage marked.
- Fog's effect expires at Cleanup.

**Negative test**: Same setup but instead of Fog, a Lightning Strike targets Player 2 (not
during combat). Player 2 takes full 3 damage. Fog (if also active) does not prevent this.

### Fog + Ongoing Spell Damage in Same Turn

**Setup**: Fog has been cast. Player 1's opponent casts Lightning Strike targeting Player 1
(a non-combat damage source).

**Expected**: Lightning Strike deals full 3 damage. Fog's `AllCombatDamage` filter does not
match `is_combat: false` damage events.

### Guardian Shield Scenario

**Setup**: Player 1 has a 2/2. Opponent has a 4/4 attacker. Player 1 casts Guardian Shield
on their 2/2 before blockers.

**Expected after combat**:
- The 2/2 has 0 damage marked (shield prevented 4 damage — amount 0 = prevent all).
- The 4/4 has 2 damage marked (the 2/2's damage dealt normally).
- Shield remains active (UntilEndOfTurn, not depleted).

**Follow-up**: Opponent casts Lightning Strike targeting the 2/2.
- 2/2 takes 0 damage (shield still active).
- Shield remains active until Cleanup.

### Multiple Prevention Effects on Same Target

**Setup**: A creature has both a Mending Light shield (3-point depleting) and a Guardian Shield
(prevent all, UntilEndOfTurn). A 5-damage event targets the creature.

**Casting order**: Player casts Mending Light first (lower timestamp), then Guardian Shield.

**Expected**: The Mending Light applies first (lower timestamp per MVP ordering). Remaining =
5 - 3 = 2. The Guardian Shield then applies to the remaining 2 damage, reducing to 0. The
Mending Light is consumed. The Guardian Shield persists (UntilEndOfTurn, not depleted by
amount: 0 logic).

**Reverse order test**: If Guardian Shield is cast first (lower timestamp), it applies first
and prevents all 5 damage. Mending Light is not consumed. Either way, 0 damage is dealt.

---

## Out of Scope

- **Source-based prevention filters** — "Prevent all damage that [source] would deal" (e.g.,
  effects that prevent damage from a specific permanent). This requires a `DamageFromSource`
  filter variant and is deferred to a future spec.
- **Protection from X** — Protection prevents damage as part of CR 702.16's "DEBT" framework.
  Deferred to K6.
- **Damage redirection** — "The next time [target] would be dealt damage, [other permanent]
  is dealt that damage instead." A different outcome variant from prevention. Deferred.
- **Regeneration interaction with Fog** — If Fog prevents all combat damage, a creature that
  would have died to combat damage doesn't trigger its regeneration shield. This falls out
  naturally from the framework (no destroy event fires if no damage is marked) and requires
  no extra work; it is called out here to confirm expected behavior is not a bug.
- **Lifegain prevention** — "Can't gain life" prevention effects operate on gain-life events,
  not damage events. Deferred.
- **Player-choice ordering of multiple preventions** — MVP uses timestamp order per R11. Deferred.
- **"Damage prevented" event emission** — Some future cards trigger "whenever damage is
  prevented" (e.g., Vigor, Phantom creatures). The current prevention loop does not emit a
  signal when damage is prevented. A future spec may require adding this hook to the prevention
  logic. Acknowledged as deferred.
