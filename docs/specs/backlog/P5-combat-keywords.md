# P5 — Combat Keywords (First Strike, Trample, Deathtouch, Lifelink)

## Overview

Echomancy currently resolves combat damage in a single simultaneous step with no
keyword modifiers. This spec introduces four evergreen combat keywords that
change how damage is assigned and applied:

- **First Strike** — deals damage in an earlier step; creatures killed before
  the regular damage step never deal their damage.
- **Trample** — excess damage beyond what is needed to kill a blocker carries
  through to the defending player.
- **Deathtouch** — any amount of damage from this creature is lethal, regardless
  of the blocker's toughness.
- **Lifelink** — damage dealt by this creature also causes its controller to gain
  that much life.

Together these four keywords represent the most impactful combat interactions in
a basic MTG-like game. They are all "always-on" static abilities (no stack,
no activation), but they change the outcome of combat in ways that players need
to understand and factor into attack/block decisions.

### Design goals

- Model each keyword as a new `StaticAbility` variant so the rest of the engine
  can query them with the existing `has_static_ability` pattern.
- First Strike requires inserting a new `FirstStrikeDamage` step between
  `DeclareBlockers` and `CombatDamage`; the regular `CombatDamage` step becomes
  "regular damage" for creatures without first strike.
- Trample changes how the damage calculation service allocates damage when an
  attacker is blocked.
- Deathtouch changes what counts as "lethal damage" inside the state-based
  actions (SBA) service.
- Lifelink is handled as a side-effect during damage application: whenever
  damage from a lifelink source is marked or dealt, the source's controller
  gains the same amount of life.

### Relationship to other systems

- Depends on the existing `StaticAbility` enum in `enums.rs`.
- Depends on the existing `calculate_all_combat_damage` function in
  `combat_resolution.rs` and the `resolve_combat_damage` helper in
  `internals.rs`.
- Depends on the existing SBA service (`state_based_actions.rs`) for lethal
  damage detection.
- First Strike requires adding a new `Step::FirstStrikeDamage` variant to the
  `Step` enum and wiring it into the step machine.
- Does NOT depend on P2 (instant-speed casting) or P4 (continuous effects),
  though those features will interact with these keywords once implemented.

---

## MTG Rules References

| Rule | Summary |
|------|---------|
| CR 702.2 | Deathtouch: any amount of damage a source with deathtouch deals is lethal. |
| CR 702.7 | First Strike: creatures with first strike deal combat damage in a separate first combat damage step before regular combat damage. |
| CR 510.4 | If at least one attacking or blocking creature has first strike, only those creatures assign damage in the first combat damage step. If none do, only one damage step occurs. |
| CR 509.1h | An attacking creature that was blocked remains "blocked" even if all blockers are removed. |
| CR 510.1c | A blocked creature with no remaining blockers assigns no combat damage (unless it has trample). |
| CR 702.15 | Lifelink: damage dealt by a creature with lifelink causes its controller to gain that much life. |
| CR 702.15b | Lifelink is a static ability; the life gain is not an event that uses the stack. |
| CR 702.19 | Trample: if a creature with trample is blocked, its controller may assign the excess damage to the defending player. |
| CR 702.19b | Before assigning excess trample damage to the player, the attacker's controller must assign at least lethal damage to each blocker. |
| CR 510.1 | In the combat damage step, each attacking and blocking creature assigns its combat damage. |
| CR 704.5g | A creature with toughness 0 or less is destroyed as a state-based action. |
| CR 704.5h | A creature with damage marked on it equal to or greater than its toughness is destroyed as a state-based action (unless it has indestructible). |
| CR 704.5h (deathtouch corollary) | Any amount of damage from a deathtouch source is considered lethal, so even 1 damage triggers destruction. |

---

## User Stories

**As a player**, I want creatures with First Strike to deal damage before
creatures without it, so that a 2/2 First Strike creature can kill a 2/2
regular creature before it deals damage back.

**As a player**, I want attackers with Trample to push excess damage through to
the defending player, so that a 6/6 blocked by a 1/1 still deals 5 damage to my
opponent.

**As a player**, I want creatures with Deathtouch to kill any creature they deal
damage to, so that a 1/1 Deathtouch creature is a credible threat against a 5/5.

**As a player**, I want to gain life equal to the damage my Lifelink creature
deals, so that combat with a Lifelink creature increases my life total whether
it hits a player or a creature.

---

## Player Experience

### First Strike

1. Player A attacks with a 2/1 creature that has First Strike.
2. Player B blocks with a 2/2 creature.
3. The game enters a new `FirstStrikeDamage` step (visible in the HUD as "First
   Strike Damage").
4. The 2/1 First Strike creature deals 2 damage to the 2/2 blocker. The blocker
   receives lethal damage (2 >= 2 toughness).
5. State-based actions fire: the 2/2 blocker is destroyed and moves to the
   graveyard.
6. The game advances to the regular `CombatDamage` step.
7. The 2/1 First Strike creature has no blockers remaining; it is now considered
   unblocked. However, per CR 509.1h, a creature that was blocked remains
   "blocked" — it does NOT deal damage to the player unless it has Trample.
   Per CR 510.1c, without Trample the 2/1 deals no damage.
8. The 2/2 blocker is already dead and deals no damage.

**Key rule clarification**: "was blocked" status persists even if the blocker
dies in the first strike step. An attacker that was blocked but whose blocker
died does not become unblocked unless it has Trample.

**Optimization rule (CR 702.7b)**: If no creature in combat has First Strike (or
Double Strike, which is out of scope), the `FirstStrikeDamage` step is skipped
entirely. The game goes directly from `DeclareBlockers` to `CombatDamage`.

### Trample

1. Player A attacks with a 6/6 creature that has Trample.
2. Player B blocks with a 1/1 creature.
3. During the `CombatDamage` step, Player A's 6/6 must assign at least 1 damage
   to the 1/1 blocker (lethal damage = 1, since the 1/1 has 1 toughness).
4. The remaining 5 damage is assigned to Player B directly.
5. Player B's life total drops by 5. The 1/1 blocker receives 1 damage (lethal)
   and is destroyed by SBAs.

**Deathtouch + Trample interaction**: A creature with both Deathtouch and Trample
only needs to assign 1 damage to a blocker to satisfy the "lethal damage"
requirement, regardless of the blocker's toughness. All remaining damage
tramples through to the player.

### Deathtouch

1. Player A attacks with a 1/1 creature that has Deathtouch.
2. Player B blocks with a 5/5 creature.
3. The 1/1 deals 1 damage to the 5/5. Normally 1 < 5 toughness, so the 5/5
   would survive.
4. Because the source has Deathtouch, 1 damage from it is lethal. The SBA check
   treats any marked damage from a Deathtouch source as lethal damage.
5. The 5/5 is destroyed. The 1/1 receives 5 damage (>= 1 toughness) and is also
   destroyed.

**SBA change required**: The current SBA service destroys creatures when
`damage_marked >= toughness`. Deathtouch requires a new flag on the damage
marking: damage from a Deathtouch source is lethal regardless of the amount. The
SBA service must know whether any damage on a creature came from a Deathtouch
source.

### Lifelink

1. Player A attacks with a 3/3 creature that has Lifelink.
2. The creature deals 3 damage to Player B (unblocked).
3. Player B's life total decreases by 3.
4. Player A's life total increases by 3 simultaneously.
5. Player A's HUD life counter visibly increases.

Lifelink also applies when the Lifelink creature is blocked:
1. Player A attacks with a 3/3 Lifelink creature.
2. Player B blocks with a 2/2.
3. The 3/3 deals 3 damage to the 2/2 blocker.
4. Even though the damage went to a creature, Player A still gains 3 life.
5. The 2/2 is destroyed by SBAs (3 >= 2 toughness).

---

## Game Rules and Mechanics

### New StaticAbility Variants

The `StaticAbility` enum gains four new variants:

- `FirstStrike`
- `Trample`
- `Deathtouch`
- `Lifelink`

All existing query patterns (`has_static_ability`) work unchanged.

### New Combat Step: FirstStrikeDamage

A new step `Step::FirstStrikeDamage` is inserted between `DeclareBlockers` and
`CombatDamage` in the turn sequence.

The step machine logic:

- When entering `FirstStrikeDamage`: check if any attacking or blocking creature
  has `FirstStrike`. If none do, skip this step immediately and advance to
  `CombatDamage`.
- During `FirstStrikeDamage`: only creatures with `FirstStrike` deal damage.
  Creatures without `FirstStrike` deal no damage in this step. Each creature
  that deals damage gets a `dealt_first_strike_damage: true` flag on its
  `CreatureSubState`. After damage is applied, SBAs fire. Creatures that die
  here do not participate in the regular `CombatDamage` step.
- During `CombatDamage`: only creatures that have NOT yet dealt damage in this
  combat (i.e. `dealt_first_strike_damage == false`) deal damage. This is
  tracked per-creature rather than re-querying the `FirstStrike` ability,
  because CR 702.7c says a creature that already dealt first strike damage
  does not deal damage again even if it loses First Strike mid-combat.
  Double Strike is out of scope — Double Strike creatures deal damage in both
  steps.
- When ALL creatures in combat have First Strike: the regular `CombatDamage`
  step still occurs (it is NOT skipped), but no creatures are eligible to deal
  damage in it since all have `dealt_first_strike_damage == true`.

**Step sequence change** (updated from current):

```
DeclareAttackers
DeclareBlockers
FirstStrikeDamage  ← NEW (may be skipped if no first strikers)
CombatDamage       ← now "regular damage step"
EndOfCombat
```

The HUD step indicator must display "First Strike Damage" for the new step.

### Trample Damage Calculation

The `calculate_all_combat_damage` function is updated to handle Trample:

For a blocked attacker with Trample:
1. Determine lethal damage for each blocker. Lethal damage is
   `max(0, blocker_toughness - damage_already_marked)`. This is critical for
   the First Strike + Trample interaction: if a blocker already took damage in
   the first strike step, less additional damage is needed. If the attacker
   also has Deathtouch, lethal damage for each blocker is 1 (or 0 if the
   blocker already received any deathtouch damage).
2. Sum the lethal damage required across all blockers (MVP: 1 blocker, so this
   is just the single blocker's lethal threshold).
3. If attacker power > total lethal damage required, the remainder is assigned
   to the defending player.
4. If attacker power <= total lethal damage required, all damage goes to the
   blocker(s) with none to the player.

For a blocked attacker WITHOUT Trample: behavior is unchanged. All damage goes
to the blocker, none to the player, even if the attacker has more power than the
blocker's toughness.

**If the blocker dies before the damage step** (e.g., removed by an instant —
relevant when P2 is implemented): a blocked attacker with Trample that has no
living blockers deals its full power to the defending player. Without Trample,
it still deals no damage (already blocked status).

### Deathtouch and SBA Integration

The `DamageAssignment` struct gains a new boolean field: `is_deathtouch`.

When a creature with `Deathtouch` deals damage to another creature, the resulting
`DamageAssignment` has `is_deathtouch: true`.

The SBA service (`find_creatures_to_destroy`) is updated:

- A creature is destroyed if: `damage_marked_this_turn >= current_toughness`
  (existing rule), OR if `has_deathtouch_damage == true` AND
  `damage_marked_this_turn > 0` (new rule).

The `CreatureSubState` (or `PermanentState`) gains a flag:
`has_deathtouch_damage: bool`. This flag is set to `true` when any damage from a
Deathtouch source is marked on the creature. It is cleared alongside
`damage_marked_this_turn` during the Cleanup step.

### Lifelink Application

Lifelink is applied in the damage application phase, not as a trigger. When
damage assigned by a Lifelink creature is applied:

- If the source creature has `Lifelink`, immediately add `damage_amount` life to
  the source creature's controller.
- This happens in the same pass as the damage application (no stack, no trigger
  window).

The `resolve_combat_damage` helper in `internals.rs` must know the controller of
each attacking/blocking creature to apply the life gain. The
`CreatureCombatEntry` struct gains the controller's player ID so this information
is available in the damage calculation service.

### Interaction Summary Table

| Scenario | Result |
|----------|--------|
| First Strike attacker vs. regular blocker | Attacker deals damage first; blocker may die before dealing its damage |
| Regular attacker vs. First Strike blocker | Blocker deals damage first; attacker may die before dealing its damage |
| Both have First Strike | Both deal damage simultaneously in the first strike step |
| Neither has First Strike | Normal simultaneous damage in regular damage step (no change) |
| Trample, unblocked | Full power to player (unchanged) |
| Trample, blocked | Lethal to blocker, remainder to player |
| Trample + Deathtouch, blocked | 1 to blocker (lethal), remainder to player |
| Deathtouch, blocked | Blocker dies regardless of toughness; both may trade |
| Lifelink attacker, unblocked | Damage to player + equal life gain for controller |
| Lifelink attacker, blocked | Damage to blocker + equal life gain for controller |
| Lifelink blocker, blocking | Damage to attacker + equal life gain for blocker controller |
| First Strike + Lifelink | Life gain occurs in the first strike damage step |
| First Strike + Trample, blocker killed in FS step | No additional damage in regular step (already dealt in FS step) |
| Trample + Lifelink | Life gain equals total damage dealt (to blocker + to player) |
| First Strike vs. Deathtouch (FS creature kills DT creature) | DT creature dies before dealing damage; First Strike survives |
| First Strike vs. Deathtouch (FS creature doesn't kill DT creature) | DT creature deals damage in regular step; both may die |

### Prerequisites / Existing Bugs

**`resolve_combat_damage` uses the wrong function**: The current implementation
in `internals.rs` calls `calculate_damage_assignments` (attacker-only) instead
of `calculate_all_combat_damage` (bidirectional). The bidirectional function
exists but is dead code. This MUST be fixed as part of P5 since First Strike
blockers must deal damage to attackers in the first strike step.

**`DamageAssignment` needs `source_id`**: Both Deathtouch (`is_deathtouch` flag)
and Lifelink (controller life gain) require knowing which creature produced the
damage. The current struct has no `source_id` field.

---

## Acceptance Criteria

- [ ] `StaticAbility` enum has `FirstStrike`, `Trample`, `Deathtouch`, and
      `Lifelink` variants.
- [ ] `Step` enum has a `FirstStrikeDamage` variant.
- [ ] The step machine inserts `FirstStrikeDamage` between `DeclareBlockers` and
      `CombatDamage`.
- [ ] If no creature in combat has First Strike, the `FirstStrikeDamage` step is
      entered but immediately exits without dealing damage or emitting a
      `StepStarted` event (or alternatively: the step is skipped in the step
      machine and never entered). Either behaviour is acceptable as long as the
      player does not see an unnecessary pause.
- [ ] `CreatureSubState` has a `dealt_first_strike_damage: bool` field, cleared
      at end of combat.
- [ ] A First Strike attacker deals damage in `FirstStrikeDamage` and gets
      `dealt_first_strike_damage = true`; it does NOT deal damage again in
      `CombatDamage` (checked by flag, not by keyword).
- [ ] A regular creature that is killed during `FirstStrikeDamage` does not deal
      damage in `CombatDamage`.
- [ ] A blocked attacker with Trample assigns lethal damage to its blocker and
      the remainder to the defending player.
- [ ] A blocked attacker without Trample assigns all damage to the blocker; no
      damage reaches the defending player regardless of the power/toughness gap.
- [ ] A blocked attacker with both Trample and Deathtouch needs to assign only 1
      damage to the blocker; remaining power goes to the defending player.
- [ ] A creature with Deathtouch that deals any damage to another creature causes
      that creature to be destroyed by SBAs, regardless of the victim's
      toughness.
- [ ] A creature with Lifelink causes its controller's life total to increase by
      the amount of damage it deals (whether to a player or a creature).
- [ ] Lifelink life gain is not a triggered ability; it occurs in the same
      damage-application pass with no stack interaction.
- [ ] The HUD step indicator displays a distinct label for the `FirstStrikeDamage`
      step.
- [ ] `resolve_combat_damage` uses bidirectional damage (blockers deal damage to
      attackers, not just attackers to blockers).
- [ ] `DamageAssignment` has `source_id` and `source_controller_id` fields.
- [ ] Creatures with 0 or less power skip damage assignment entirely.
- [ ] All existing combat tests continue to pass (no regression).
- [ ] New unit tests cover each keyword in isolation, the Deathtouch + Trample
      interaction, and the First Strike vs Deathtouch interaction.

---

## Out of Scope

| Feature | Reason deferred |
|---------|-----------------|
| Double Strike | Too complex for this pass; requires dealing damage in both first strike and regular steps. Spec separately when First Strike is stable. |
| Indestructible | Requires distinguishing "lethal damage" from "destroy" effects. Deathtouch + Indestructible is a known edge case. Spec separately. |
| Protection | Complex targeting and damage prevention interaction; separate spec. |
| Multiple blockers per attacker | Trample damage ordering with multiple blockers requires the active player to order blockers and assign damage individually. Currently MVP limits to 1 blocker per attacker. |
| Menace (must be blocked by two or more creatures) | Blocking restriction keyword; can be added cheaply once multiple blockers are supported. |
| Combat tricks / instants during combat | Priority windows before and after damage steps (P2) are not yet implemented; instants cannot currently be cast in response to First Strike damage. |
| Non-combat Lifelink | Lifelink applies to all damage a creature deals, including damage from activated abilities. This spec covers combat damage only; non-combat sources are deferred. |
| CLIPS rules for these keywords | This spec covers the domain model and combat resolution service. If CLIPS takes over combat resolution in the future, the CLIPS rules are a separate task. |
