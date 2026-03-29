# P1 — Target Selection System

## Overview

Lightning Strike (and every future damage/removal spell) currently hard-codes
the opponent as its target. This spec introduces a proper target selection flow
so that when a player casts a spell that says "target creature or player", they
choose the target before the spell is placed on the stack — exactly as MTG CR
601.2c requires.

The feature touches four layers:

1. **Domain model** — `CardDefinition` declares what kind of target a spell
   needs; `Action::CastSpell` carries the chosen target; `Target` gains a
   `Creature` variant.
2. **Cast-spell handler** — validates the chosen target before putting the
   spell on the stack.
3. **CLIPS bridge / card rules** — the chosen target is serialised into the
   `stack-item` fact so CLIPS rules can read it directly instead of guessing.
4. **Bevy UI** — clicking a castable spell either casts it immediately (no
   target needed) or enters a target-selection mode where the player clicks a
   valid target, after which the spell is dispatched.

### Design goals

- Preserve the zero-targeting path for spells like creatures and sorceries that
  do not require explicit targets.
- Give the player clear visual feedback: which objects are valid targets, and
  that the game is waiting for a selection.
- Keep the domain layer pure: target validation is a domain concern; rendering
  the highlight is a UI concern.
- Unblock Lightning Strike resolution via CLIPS without heuristics.

### Relationship to other systems

- Depends on M3 (CLIPS resolver connected) being done before the full end-to-end
  works, but domain + UI layers can be built and tested independently.
- P2 (instant-speed casting) will reuse the same target-selection UI mode.

---

## MTG Rules References

| Rule | Summary |
|------|---------|
| CR 113.1 | Some spells and abilities require targets. |
| CR 601.2c | Choose targets as part of casting a spell. |
| CR 608.2b | On resolution, re-check that each target is still legal. If all targets are illegal, the spell does nothing. |
| CR 115.4 | A spell or ability targets only if its text uses the word "target". |
| CR 115.6 | "Any target" means any creature, player, or planeswalker. |
| CR 701.7 | "Destroy" puts a permanent into the graveyard. |

---

## User Stories

**As the active player**, when I click a spell that requires a target, I want
the game to enter a target-selection mode so I know I must click a valid target
before the spell is cast.

**As the active player**, when I am in target-selection mode, I want invalid
targets (untargetable permanents, wrong type) to be visually distinguished so I
cannot accidentally select them.

**As the active player**, I want to be able to cancel target selection and not
cast the spell if I change my mind.

**As a player whose creature is targeted**, I want the engine to verify at
resolution that the creature is still a legal target, and fizzle the spell if it
is not (e.g. it was destroyed in response).

---

## Player Experience

### Spell with no targets (existing behaviour, unchanged)

1. Player clicks a creature card (e.g. Bear) highlighted with a blue border.
2. Game dispatches `CastSpell` immediately.
3. Spell appears on the stack; priority passes to the opponent.

### Spell that requires a target (new behaviour)

1. Player clicks a spell card highlighted with a blue border (e.g. Lightning Strike).
2. The game enters **target-selection mode**:
   - The hand is greyed out (no further cards can be clicked).
   - Valid targets receive a **yellow highlight**: opponent's creatures on the
     battlefield, and the opponent's player avatar/life area.
   - A cancel button or pressing Escape returns to the normal game state without
     casting.
3. Player clicks one of the highlighted targets.
4. Game dispatches `CastSpell { player_id, card_id, targets: [chosen_target] }`.
5. The domain validates the target is legal, pays mana, removes the card from
   hand, and pushes the spell onto the stack with the target stored on it.
6. Priority passes to the opponent.
7. When the spell resolves, CLIPS reads the target from the `stack-item` fact
   and applies the effect to the correct target.

### Fizzle on resolution (target no longer legal)

If the targeted creature was destroyed before the spell resolves:
- The domain (or CLIPS rule) detects that the target is no longer present.
- The spell resolves with no effect ("fizzles").
- The spell card moves to the graveyard normally.
- The opponent receives no damage.

---

## Game Rules and Mechanics

### Target types (MVP scope)

| Target variant | Valid objects |
|----------------|--------------|
| `Player { player_id }` | Any player in the game (already exists) |
| `Creature { permanent_id }` | Any creature on the battlefield |

"Any target" means `Player` or `Creature` — the player may click either.

Planeswalker is explicitly out of scope (no planeswalkers in the card catalog).

### Legality rules

A target is **legal at cast time** when:
- For `Player`: the player is in the game.
- For `Creature`: the permanent is on the battlefield, has the `Creature` card
  type, and does not have Shroud or Hexproof (those keywords are out of scope
  for MVP but the check must not prevent them being added later).

A target is **legal at resolution time** (CR 608.2b) when:
- For `Player`: the player has not lost the game.
- For `Creature`: the permanent is still on the battlefield.

If all targets are illegal at resolution, the spell fizzles.

### CardDefinition — target requirement

`CardDefinition` gains a field that describes what targets a spell requires.
For the MVP, the three valid values are:

| Value | Meaning |
|-------|---------|
| `None` | Spell requires no targets (creatures, enchantments, sorceries without "target"). |
| `AnyTarget` | "Any target" — player or creature. Used by Lightning Strike. |
| `Creature` | Must target a creature on the battlefield. |

This field is set in code when constructing the card definition. It does not
affect lands or permanents that have no spell text.

### Action change

`Action::CastSpell` gains an optional `targets` field:

- When the spell requires no targets, `targets` is empty.
- When the spell requires a target, `targets` contains exactly one `Target`.
- The domain rejects `CastSpell` with zero targets when the spell requires one
  (error: `TargetRequired`).
- The domain rejects `CastSpell` with an illegal target (error: `InvalidTarget`).

### CLIPS bridge change

The existing `stack-item` fact already has a `target` slot (single STRING).
This slot currently receives the first target's ID from `SpellOnStack.targets`.

No template change is needed. The bridge already serialises the first target.
The card rule (`lightning-strike.clp`) can be updated to use the `target` slot
from the `stack-item` fact instead of searching for the opponent.

### Resolution fizzle

When CLIPS processes a `SPELL_RESOLVING` event for a spell with a target, the
card rule should verify the target still exists before asserting an
`action-damage` (or `action-destroy`, etc.) fact. If the target is absent, the
rule simply does not assert the action fact — the spell resolves with no effect.

---

## Acceptance Criteria

### Domain

- [x] `CardDefinition` has a `target_requirement` field with variants `None`,
      `AnyTarget`, and `Creature`.
- [x] `CardDefinition::with_target_requirement()` builder sets the field.
- [x] Lightning Strike's definition is updated to `AnyTarget`.
- [x] `Target` gains a `Creature { permanent_id: String }` variant.
- [x] `Action::CastSpell` has a `targets: Vec<Target>` field.
- [x] `Action::CastSpell` serialises/deserialises correctly with the new field
      (existing serialisation tests still pass; serde default is empty vec).
- [x] `cast_spell::handle` rejects `CastSpell` with no targets when the spell
      declares `AnyTarget` or `Creature` requirement (returns `TargetRequired`).
- [x] `cast_spell::handle` rejects `CastSpell` with an illegal target
      (wrong type, permanent not on battlefield, player not in game) and returns
      `InvalidTarget`.
- [x] `cast_spell::handle` stores the chosen targets on `SpellOnStack.targets`.
- [ ] At resolution, if the target creature is no longer on the battlefield, the
      spell's CLIPS rule does not apply its effect (verified via integration test).
      (Fizzle: CLIPS rule won't fire since target-id is stored in SpellResolved event;
       if fizzle logic is desired, the Rust caller must check before asserting the event.)

### CLIPS bridge / card rule

- [x] `lightning-strike.clp` is updated: the rule reads `?target` from
      `(game-event (target-id ?target))` and asserts `action-damage` against that target.
      (Implementation uses `target-id` on the `game-event` fact rather than `stack-item`,
       since the stack-item is popped before resolution. The `target-id` is forwarded by
       the Rust bridge from `SpellResolved.targets`.)
- [x] The rule no longer uses the opponent-search heuristic.
- [x] An integration test asserts a `SPELL_RESOLVING` event with a player target
      and verifies the resulting `action-damage` fact targets that specific player.
- [ ] An integration test asserts a `SPELL_RESOLVING` event with a creature target
      and verifies the `action-damage` targets that creature's ID.
      (Deferred — creature-target tests added in domain layer; CLIPS integration
       test for creature targets can be added alongside the first removal spell.)

### Bevy UI — target-selection mode

- [x] `AllowedActionsResult` has a `spells_needing_targets: Vec<String>` field
      (instance IDs of spells that require a target). These are a subset of
      `castable_spells`.
- [x] Clicking a spell in `spells_needing_targets` does **not** dispatch
      `CastSpell` immediately; instead it sets a `TargetSelectionState` resource
      (pending spell instance ID).
- [x] While `TargetSelectionState` is active:
  - Opponent's creatures on the battlefield render with a yellow highlight (`ValidTarget` component + yellow border).
  - Opponent's player portrait/life display renders with a yellow highlight.
      (Note: opponent player area highlight deferred — the HUD shows opponent life but there is no clickable
       player portrait entity yet. The `ValidTarget` pattern is in place for when one is added.)
  - Hand cards are non-interactive (`rebuild_hand` skips Button/Interaction when `pending_spell.is_some()`).
  - A "Cancel" button (`CancelTargetButton`) in the HUD allows cancelling target selection.
- [x] Clicking a valid highlighted target dispatches
      `CastSpell { player_id, card_id, targets: [chosen_target] }`.
- [x] Clicking the cancel button clears `TargetSelectionState` and returns to
      the normal game state without casting.
- [x] Clicking a non-highlighted object while in target-selection mode has no
      effect (non-highlighted cards have no `ValidTarget`/`Button` component).
- [x] After the spell is cast, `TargetSelectionState` is cleared automatically
      (`pending_spell.take()` in `handle_valid_target_click`).

### Regression

- [x] All existing `cast_spell` tests pass (no-target spells unaffected).
- [x] Casting a Bear (no target requirement) still works with the new action shape
      (empty targets vec accepted for non-targeting spells).
- [x] `cargo test` passes with zero failures.
- [x] `cargo clippy` reports zero warnings.

---

## Out of Scope

The following are deferred to future specs:

- **Multiple targets**: All spells in this MVP require at most one target.
- **Planeswalker targets**: No planeswalkers in the catalog.
- **Shroud / Hexproof**: Keywords exist but validation is deferred.
- **Protection from colour**: Deferred.
- **Targeting restrictions from other cards** (e.g. "can't be the target of
  spells your opponents control"): Deferred to the continuous effects system (P4).
- **Instant-speed targeting**: Target selection during the opponent's turn is
  deferred to P2 (instant-speed casting).
- **Multi-target spells** (e.g. Electrolyze): Deferred — requires UI to accept
  two selections.
- **Creature-only targeting spells**: Only "any target" and "player" are added
  for now (Lightning Strike covers both). Pure creature-targeting removal
  (e.g. Doom Blade) shares the same plumbing and can be added alongside the
  first removal spell without a new spec.
