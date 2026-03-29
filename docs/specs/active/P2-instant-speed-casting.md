# P2: Instant-Speed Casting

## Overview

This spec covers replacing the `auto_resolve_stack` shortcut with genuine priority-based stack resolution, enabling players to cast instants in response to spells and during combat steps. It is the single biggest interactive upgrade to the current engine: spells no longer auto-resolve the moment they are cast.

**Design goal**: Players experience the real MTG priority loop — cast a spell, opponent gets a window to respond, both must actively pass priority for anything to resolve.

**Relationship to other systems**:
- Depends on the priority system already implemented in `pass_priority.rs`
- Replaces the `auto_resolve_stack` automation in `handle_game_actions`
- Extends `compute_castable_spells` in `legal_actions.rs` to include instant-speed cards at non-sorcery windows
- Pairs with U1 (Stack Display) from the UI backlog — players need to see the stack

---

## User Stories

**As the active player**, I want to cast an instant after my creature is placed on the stack so that I can further develop my board before the opponent responds.

**As the non-active player**, I want to cast Lightning Strike in response to my opponent's creature being cast so that I can deny them the tempo while their mana is committed.

**As either player during combat**, I want to cast an instant after attackers are declared (but before damage) so that I can change the combat outcome.

**As a player with no instants in hand**, I want the game to auto-advance past priority windows I cannot meaningfully use so that I am not clicking "Pass Priority" dozens of times per turn.

---

## Player Experience

### Scenario A: Casting an instant in response to a spell

1. Player 1 (active, FirstMain) casts a creature. The creature goes on the stack. Per CR 117.3c, Player 1 retains priority. Player 1 passes. Priority goes to Player 2.
2. The UI switches perspective to Player 2. Player 2 sees the stack with the creature on it, and their instants in hand are highlighted as castable.
3. Player 2 casts Lightning Strike targeting Player 1. Lightning Strike goes on top of the stack. Priority passes to Player 1.
4. Player 1 has no response and presses "Pass Priority." Priority passes back to Player 2.
5. Player 2 also has nothing to add and presses "Pass Priority." Both players have passed in succession — Lightning Strike resolves.
6. After resolution, active player (Player 1) gets priority. Player 1 passes again. Player 2 passes again. The creature resolves.

### Scenario B: Casting an instant during DeclareAttackers

1. Player 1 declares an attacker. Priority is given to Player 2 (defending player).
2. Player 2 sees their castable instants highlighted. Player 2 casts a removal spell targeting the attacker.
3. Priority passes back to Player 1 (active player).
4. Player 1 passes. Player 2 passes. The removal resolves, destroying the attacker.
5. Priority returns to Player 1. Player 1 passes. Player 2 passes. Combat continues with no attackers.

### Scenario C: Auto-pass when nothing can happen

1. Player 1 casts a creature. Player 2 has no instants in hand and no mana.
2. The game detects Player 2 cannot act and auto-passes their priority.
3. Player 1 receives priority back. Player 1 also has nothing to add. Auto-pass fires.
4. Both players have auto-passed — the creature resolves. No clicking required.

### Priority button behavior

- **"Pass Priority"** button is always visible when the player has priority.
- Pressing it records the player's pass and transfers priority to the opponent.
- When both players have passed in succession (with no new spell cast in between), the top of the stack resolves.
- When the stack is empty and both players pass, the step advances.

---

## Game Rules and Mechanics

### MTG Comprehensive Rules (relevant subset)

| Rule | Description |
|------|-------------|
| CR 116.1 | A player who has priority may cast spells, activate abilities, or take special actions |
| CR 117.3a | Active player receives priority at beginning of most steps/phases. No priority during Untap. Usually no priority during Cleanup. |
| CR 117.3c | After a player casts a spell, **that same player** receives priority (not the opponent) |
| CR 116.3b | After each resolution, the active player receives priority |
| CR 116.4 | Both players must pass priority in succession (with no intervening action) for the top of the stack to resolve |
| CR 304.5 | Sorceries may only be cast at sorcery speed |
| CR 601.2 | Casting a spell requires having priority |

### Instant-speed windows

Instants (and permanents with Flash) can be cast whenever the player has priority, including:

| Situation | Who has priority | Can cast instant? |
|-----------|-----------------|-------------------|
| Untap | Nobody | No |
| Upkeep | Active player | Yes |
| Draw (after draw action) | Active player | Yes |
| Main phase, empty stack | Active player | Yes (also sorcery speed) |
| Spell on stack after cast | Caster (CR 117.3c), then opponent on pass | Yes |
| BeginningOfCombat | Active player | Yes |
| DeclareAttackers (after declared) | Active player, then opponent | Yes |
| DeclareBlockers (after declared) | Active player, then opponent | Yes |
| FirstStrikeDamage (after damage) | Active player | Yes |
| CombatDamage (after damage) | Active player | Yes |
| EndOfCombat | Active player | Yes |
| EndStep | Active player | Yes ("on your end step...") |
| Cleanup | Nobody (unless triggers fire) | No |

Sorcery-speed spells (creatures without Flash, sorceries) remain restricted to: active player's turn, main phase (First or Second), empty stack.

### Priority reset on new spell

When any player casts a spell or activates an ability, the "both players passed" counter resets. Both must pass again before the newly added item (now the top of the stack) can resolve.

### Stack order and resolution

The stack resolves in LIFO order. The most recently cast spell resolves first. This is already implemented in `resolve_top_of_stack()` — nothing changes here.

### Auto-pass heuristic

To avoid requiring manual clicks for every priority transfer when neither player can act:

A player's priority is auto-passed when ALL of the following are true:
1. The player has no instants (or Flash creatures) in hand.
2. The player has no mana to cast any instant-speed card they hold.
3. There are no activated abilities available that could meaningfully be used (mana abilities are the common case, but those are relevant only if there are instants to cast).

If both players qualify for auto-pass simultaneously after a spell resolves, the active player gets priority first (per CR 116.3b), auto-passes, then the opponent auto-passes, and the next stack item resolves (or the step advances if the stack is empty).

Per CR 117.3a, players receive priority at most steps. The only steps where
no player receives priority are **Untap** and **Cleanup** (unless triggered
abilities fire during Cleanup).

`is_non_interactive_step` is updated to only include Untap and Cleanup.
All other steps (Upkeep, Draw, BeginningOfCombat, EndOfCombat, EndStep)
become interactive — meaning players receive priority and can cast instants.
The auto-pass heuristic handles the common case where neither player has
actions, keeping the experience smooth.

`perform_step_advance` must assign priority to the active player at all
interactive steps (not just main phases as it does today).

### Castable spells in `compute_legal_actions`

`compute_castable_spells` currently returns only sorcery-speed-legal cards. It must be extended:
- If the player has priority and holds an instant (or Flash card), that card is castable regardless of step or stack depth, subject to mana availability.
- The existing sorcery-speed path is unchanged.
- `compute_spells_needing_targets` must apply to both sorcery-speed and instant-speed castable spells.

### UI perspective during priority windows

The existing `resolve_ui_player_id` logic handles DeclareBlockers (defending player drives). This logic must be extended to cover any step where the non-active player holds priority (i.e., whenever a spell on the stack gives the opponent a response window):

- If the priority holder differs from the active player, show the priority holder's perspective.
- Exception: during DeclareAttackers and DeclareBlockers, the existing rules for those steps apply first.

---

## Scope of Changes

### Domain (echomancy-core)

**Remove `auto_resolve_stack` from the Bevy update loop.** The function itself can remain in `automation.rs` for tests or future use, but `handle_game_actions` must stop calling it after every action.

**Extend `compute_castable_spells`** to include instant-speed cards when the player has priority outside of sorcery-speed windows.

**Add `compute_auto_pass_eligible`** (new function in `legal_actions.rs`): returns `true` if the player has priority but no legal instant-speed action available (no instants in hand, no mana for any held instant). The Bevy layer calls this to decide whether to auto-pass.

### Bevy UI (echomancy-bevy)

**Remove the `auto_resolve_stack` call** from `handle_game_actions` in `systems.rs`.

**Add auto-pass logic** in `handle_game_actions`: after every action, check both players with `compute_auto_pass_eligible`. Auto-pass each eligible player in priority order until either a player is not eligible (they have a meaningful choice) or the stack resolves / step advances naturally.

**Update `resolve_ui_player_id`** to show the priority holder's perspective whenever priority differs from the active player (not only during DeclareBlockers).

**Highlight instant-speed castable cards** in the hand UI at all priority windows, not only during main phase. The `castable_spells` list from `AllowedActionsResult` already drives hand highlighting — extending `compute_castable_spells` is sufficient.

**"Pass Priority" button** must be visible and active whenever the player holds priority (already the case for the active player; must work equally for the non-active player when they hold priority after a spell is cast).

---

## Acceptance Criteria

- [ ] Player 2 can cast Lightning Strike targeting Player 1 in response to Player 1 casting a creature during Player 1's main phase.
- [ ] After Player 2 casts in response, Player 1 receives priority and can cast again or pass.
- [ ] Lightning Strike resolves before the creature when both players pass in the correct order.
- [ ] The creature resolves after, and lands on the battlefield.
- [ ] During DeclareAttackers, the defending player (Player 2) can cast an instant targeting the attacking creature before damage.
- [ ] During DeclareBlockers, the active player (Player 1) can cast an instant before damage.
- [x] Instants in hand are visually highlighted (castable) whenever the player has priority, not only during main phase.
- [ ] When Player 2 has no instants and no mana, the game auto-passes their priority without requiring a click.
- [ ] When both players have no instants, spells still resolve automatically (same user-visible behaviour as before, but through legitimate double-pass rather than `auto_resolve_stack`).
- [ ] The UI shows the priority holder's perspective whenever the priority holder differs from the active player.
- [x] After casting a spell, priority returns to the **caster** (CR 117.3c), not the opponent.
- [ ] A player can cast two instants in a row without the opponent receiving priority in between (by retaining priority after the first cast).
- [ ] Stack depth >= 3 works correctly (P1 casts creature, P2 responds, P1 responds to the response).
- [x] Players receive priority at Upkeep, Draw, BeginningOfCombat, EndOfCombat, and EndStep.
- [x] Only Untap and Cleanup are truly non-interactive (no priority).
- [x] Sorcery-speed cards (creatures without Flash, sorceries) remain restricted to main phase / empty stack / active player's turn.
- [ ] "Pass Priority" button works correctly from both players' perspectives at any step.

---

## Out of Scope

- **U1 Stack Display**: Showing the stack visually as a list of cards is a separate UI task (U1 in the backlog). The priority loop works without it; the stack panel is a polish concern.
- **U6 Hotseat transition screen**: A "Pass to Player 2" overlay between priority windows is a separate UI task. Players sharing a screen must manage the physical handoff themselves for now.
- **Activated abilities as responses**: Tapping a creature for an activated ability in response to a spell is theoretically legal but not in scope here. Only instant-speed spell casting is covered.
- **Triggered ability windows**: Priority windows that open due to triggered abilities resolving are not covered by this spec.
- **Counterspells**: Counter mechanics (targeting a spell on the stack) are not introduced here. A counterspell card would work with the priority system implemented here, but the card definition and CLIPS rule are out of scope.
- **Mana abilities during response**: Players can already tap lands for mana whenever they have priority. This remains unchanged and is not considered a new feature here.
- **Flash creatures on opponent's turn**: Flash is already implemented in `spell_timing.rs`. This spec does not change Flash rules, but the `compute_castable_spells` extension will make Flash cards correctly appear as castable during the opponent's turn.
