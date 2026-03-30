# G5 — Mulligan System (Vancouver Mulligan)

## Overview

Echomancy currently starts every game with both players drawing 7 cards and
immediately entering P1's First Main phase. This spec introduces a **Mulligan
phase** that runs before Turn 1, giving P1 (the human player) the ability to
redraw an opening hand they are unhappy with. The bot (P2) always keeps its
opening hand with no evaluation.

The rule system implemented is the **Vancouver Mulligan**, which is the current
MTG Standard and is defined in CR 103.4–103.5. Each player may mulligan any
number of times. After all players have kept, any player who mulliganed must put
one card on the bottom of their library per mulligan taken (called the "partial
Paris" or "free mulligan" cleanup step). The scry-1 bonus that Vancouver
introduced is deferred to keep MVP UI scope manageable.

### Design Goals

- Give P1 a chance to redraw a non-functional opening hand before the game
  starts.
- Keep P2 logic trivial (always keep) so no AI decision-making is needed.
- Introduce a distinct Bevy `AppState` variant for the mulligan phase, keeping
  it fully isolated from turn gameplay.
- Keep the UI simple: a hand display, a "Keep" button, a "Mulligan" button, and
  (if P1 mulliganed at least once after keeping) a card-selection mode for
  putting cards back.

### Relationship to Other Systems

- The Mulligan phase runs entirely before the first turn. It does not interact
  with the stack, priority, or CLIPS rules engine.
- It does touch the Library zone (shuffle) and Hand zone (draw / put back).
- It adds a new `GamePhase::Mulligan` state to the Bevy `AppState` (or
  equivalent state enum), parallel to the existing `InGame` state.
- The `Game::start()` constructor in `echomancy-core` will need to be updated
  (or a new initialisation command added) so that the game begins in a Mulligan
  state instead of immediately advancing to `FirstMain`.

---

## MTG Rules References

| Rule | Summary |
|------|---------|
| CR 103.4 | Each player draws a starting hand of seven cards. |
| CR 103.4a | A player who does not wish to keep their opening hand may take a mulligan: shuffle the hand back, draw seven cards, and begin the process again. |
| CR 103.4b | Starting with the first player, each player may mulligan until all players have kept. |
| CR 103.4c | After all players keep, each player who took one or more mulligans puts cards from their hand on the bottom of their library equal to the number of times they mulliganed. |
| CR 103.5 | After the process above, if a player's opening hand has fewer cards than their starting hand size, that player may look at the top card of their library and put it on the top or bottom (scry 1). This spec defers that step. |

---

## User Stories

**As P1**, I want to see my opening hand before the game begins and decide
whether it is good enough to play, so I am not forced into an unwinnable game
from the start.

**As P1**, if I mulligan, I want to draw a new 7-card hand and decide again,
knowing I will have to put one card back later, so I can trade hand size for
quality.

**As P1**, after I keep a hand that I mulliganed into, I want clear instructions
on how many cards to put back and a simple click-to-select interface, so I can
complete the mulligan process before the game starts.

**As a game designer**, I want P2 to always keep so the bot requires zero
decision logic for the mulligan phase, keeping implementation simple.

---

## Player Experience

### Entering the Mulligan Phase

1. The game window loads. Instead of the battlefield and HUD, the player sees a
   **Mulligan Screen**.
2. P1's opening hand (7 cards) is displayed in the centre of the screen. Each
   card shows its name, type, cost, and coloured border (same visual style as
   the in-game hand display).
3. Below the hand, the player sees two buttons: **"Keep"** and **"Mulligan"**.
4. A status label reads: "Opening hand — keep or mulligan?"
5. P2's mulligan decision is resolved silently and immediately: P2 always keeps.
   No visual feedback is needed for P2's decision.

### Taking a Mulligan

1. P1 clicks **"Mulligan"**.
2. The cards in P1's hand animate out (or simply disappear for MVP — animation
   is out of scope).
3. P1's hand is shuffled back into the library and 7 new cards are drawn.
4. The new hand is displayed.
5. The mulligan counter for P1 increments by 1. The status label updates to
   reflect how many cards P1 will have to put back if they keep now:
   - After 1 mulligan: "You will put 1 card on the bottom if you keep."
   - After 2 mulligans: "You will put 2 cards on the bottom if you keep."
6. The **"Mulligan"** button label updates to show the current hand size that
   P1 would play to: "Mulligan (to 6)", "Mulligan (to 5)", etc.
7. The **"Keep"** button remains available at all times.

### Keeping After Zero Mulligans

1. P1 clicks **"Keep"** without having mulliganed.
2. The Mulligan phase ends immediately.
3. The game transitions to Turn 1, P1's Beginning phase (Untap step).
4. Gameplay proceeds as normal.

### Keeping After One or More Mulligans

1. P1 clicks **"Keep"** after having taken N mulligans.
2. The two buttons disappear and are replaced by the instruction:
   "Put N card(s) on the bottom of your library. Click a card to send it down."
3. P1's hand remains displayed. Each card is clickable.
4. P1 clicks a card. It is removed from the hand display and placed on the
   bottom of the library. A running counter updates: "Put 2 more cards on the
   bottom." (decrements with each click.)
5. P1 repeats until N cards have been sent to the bottom.
6. Once the required number of cards has been placed, the Mulligan phase ends
   automatically (no extra confirmation button needed).
7. The game transitions to Turn 1, P1's Beginning phase (Untap step).

---

## Domain Model Changes (echomancy-core)

### New MulliganState Value Object

The game domain needs to track mulligan progress for each player. A new value
object `MulliganState` holds:

- Which players have kept (boolean per player).
- How many mulligans each player has taken (count per player).
- How many cards each player still needs to put back (decrements as cards are
  sent to the bottom).

This state is only relevant during the Mulligan phase and can be discarded once
the phase ends (or kept as historical metadata — implementer's choice).

### New Commands / Actions

The following player actions must be expressible as commands that `game.apply()`
can dispatch:

| Command | Description |
|---------|-------------|
| `MulliganDecision::Keep { player_id }` | Player decides to keep their current hand. Sets "has kept" flag. If player has N mulligans outstanding, transitions the player to the "put cards back" sub-step. |
| `MulliganDecision::Mulligan { player_id }` | Player shuffles their hand into the library, draws 7 new cards, and increments their mulligan count. |
| `MulliganDecision::PutCardOnBottom { player_id, card_instance_id }` | Moves a specific card from the player's hand to the bottom of their library. Decrements the cards-to-put-back counter. When the counter reaches zero, the player's mulligan is complete. |

### Game Initialisation Change

Currently `Game::start()` draws 7 cards for each player and sets the active
step to `FirstMain`. After this spec:

1. `Game::start()` draws 7 cards for each player as before.
2. The game is set to a new `phase: GamePhase::Mulligan` (or equivalent field).
3. P2's mulligan decision is applied immediately and automatically during
   initialisation (P2 always keeps with 0 mulligans, so no cards go back and
   the initialisation path for P2 is trivial).
4. The game does NOT advance to `FirstMain` until P1 completes their mulligan
   decision.

> Note: Whether `GamePhase::Mulligan` is a new field on `Game`, a new variant of
> an existing enum, or a new Bevy `AppState` variant is an implementation
> decision. The spec only requires that the domain can represent "we are in
> mulligan, not yet in turn play."

### Library Shuffle on Mulligan

When a `MulliganDecision::Mulligan` command is applied:

1. All cards in the player's hand move to the library.
2. The library is shuffled (same shuffle behaviour as `Game::start()`).
3. 7 cards are drawn from the top of the library to the player's hand.

The library order after the shuffle is non-deterministic (seeded RNG, same as
existing shuffle). Tests that exercise the mulligan flow should use a seeded RNG
to make draws deterministic.

### Put Cards Back: Order on the Bottom

When `PutCardOnBottom` commands are applied, each card is placed at the very
bottom of the library (position 0 if the library is stored with index 0 = bottom
and top = last index, or equivalent). Cards put back by multiple calls are
stacked at the bottom in the order they are placed (first put = deeper in the
library).

The player does NOT know what order the cards are in after being placed (the
library is a hidden zone). The order is well-defined for game correctness but
need not be surfaced in the UI.

---

## UI Changes (echomancy-bevy)

### New Bevy AppState: `Mulligan`

A new `AppState::Mulligan` variant (or equivalent game phase state) is added.
The game starts in this state. The `InGame` state starts only after the Mulligan
phase is complete.

All existing in-game UI (battlefield, HUD, hand) is hidden during `Mulligan`
state (use `StateScoped` for cleanup). Only the Mulligan-specific UI is visible.

### MulliganPlugin

A new `MulliganPlugin` handles all mulligan-phase rendering and input. It
registers:

- A system that spawns the Mulligan screen on entering `AppState::Mulligan`.
- A system that renders P1's current hand as clickable cards.
- A system that renders the Keep/Mulligan buttons (hidden during put-back phase).
- A system that renders the put-back instruction and counter (visible only during
  put-back phase).
- A system that handles card click events during the put-back phase and
  dispatches `PutCardOnBottom` actions.
- A system that watches for mulligan completion and transitions to `AppState::InGame`.

### GameSnapshot Extension

The `GameSnapshot` (player-relative view) must expose:

- Whether the game is currently in the Mulligan phase.
- P1's current hand (already available in existing snapshot).
- P1's current mulligan count.
- How many cards P1 still needs to put on the bottom (0 if not in put-back
  sub-step).

The Bevy UI reads these fields from the snapshot to drive rendering decisions.

### Buttons and Labels

| Game Sub-State | Keep Button | Mulligan Button | Instruction Label |
|----------------|-------------|-----------------|-------------------|
| First decision (0 mulligans taken) | "Keep" | "Mulligan" | "Opening hand — keep or mulligan?" |
| After 1 mulligan | "Keep" | "Mulligan (to 6)" | "You will put 1 card on the bottom if you keep." |
| After 2 mulligans | "Keep" | "Mulligan (to 5)" | "You will put 2 cards on the bottom if you keep." |
| Put-back phase (N cards remaining) | hidden | hidden | "Put N card(s) on the bottom. Click a card." |
| Put-back phase (0 cards remaining) | hidden | hidden | (phase ends automatically) |

---

## Edge Cases

| Scenario | Expected Behaviour |
|----------|--------------------|
| P1 mulligans 7 times (down to 0) | P1 must put 7 cards back. But drawing 7 and putting 7 back means P1 starts with an empty hand. This is legal per MTG rules. The game should not prevent it. |
| P1 clicks "Put Card on Bottom" with 0 cards remaining | The action is rejected (no-op or ignored); the phase has already ended. |
| P1 tries to put back a card not in their hand | The command is rejected with an appropriate error. The UI should only present legal cards. |
| P1 mulligans back to a 7-card hand identical to the previous one | No special handling; the game continues normally. The shuffle makes this extremely unlikely but not impossible. |
| P2 keeps immediately | P2's keep is applied during `Game::start()` initialisation. P2 never presents any UI. From P1's perspective, P2's hand is already decided when the Mulligan screen appears. |
| Player attempts to use Keep/Mulligan buttons during put-back phase | The buttons are hidden / non-interactive during put-back. The UI does not dispatch `Keep` or `Mulligan` commands in this state. |
| Network / disconnect (future) | Out of scope for MVP; no networking exists yet. |

---

## Acceptance Criteria

### Domain (echomancy-core)

- [x] A `MulliganDecision::Keep` command for P1 with 0 mulligans transitions the
      game to Turn 1 (active step becomes `Untap`).
- [x] A `MulliganDecision::Mulligan` command shuffles P1's hand back into the
      library, draws 7 new cards, and increments P1's mulligan count by 1.
- [x] After P1 keeps with N mulligans (N > 0), exactly N `PutCardOnBottom`
      commands must be applied before the game transitions to Turn 1.
- [x] A `PutCardOnBottom` command moves the specified card to the bottom of P1's
      library and decrements the cards-to-put-back counter.
- [x] When the cards-to-put-back counter reaches 0, the game automatically
      transitions to Turn 1.
- [x] P2 always keeps immediately during `Game::start_with_mulligan()` with a
      mulligan count of 0, no cards put back, and no external command required.
- [x] `PutCardOnBottom` with a card not in the player's hand returns an error.
- [x] P1 can mulligan 7 times and then keep, resulting in an empty hand at the
      start of Turn 1 (all 7 cards put back).
- [x] `game.export_state()` exposes the mulligan state (count per player,
      cards-to-put-back per player, whether each player has kept).
- [x] Unit tests cover: keep with 0 mulligans, keep with 1 mulligan + put back,
      keep with 2 mulligans + put back, mulligan 7 times, invalid put-back card.

### UI (echomancy-bevy)

- [x] The game starts in the Mulligan screen, not the battlefield.
- [x] P1's opening hand (7 cards) is displayed with name, type, cost, and
      coloured border.
- [x] "Keep" and "Mulligan" buttons are visible and clickable on the Mulligan
      screen.
- [x] Clicking "Mulligan" replaces the displayed hand with a new 7-card hand.
- [x] After 1 mulligan, the Mulligan button label reads "Mulligan (to 6)"; after
      2 mulligans it reads "Mulligan (to 5)"; and so on.
- [x] After 1 mulligan, the instruction label informs P1 they will put 1 card on
      the bottom if they keep.
- [x] Clicking "Keep" with 0 mulligans transitions directly to the battlefield
      and begins Turn 1.
- [x] Clicking "Keep" with N mulligans hides the Keep/Mulligan buttons and shows
      the put-back instruction with the correct count.
- [x] Clicking a card during the put-back phase removes it from the hand display
      and decrements the counter.
- [x] When the put-back counter reaches 0, the Mulligan screen disappears and
      the battlefield appears (Turn 1 has begun).
- [x] All existing gameplay tests and UI behaviour remain unaffected once the
      game transitions out of the Mulligan phase.

---

## Out of Scope

| Feature | Reason Deferred |
|---------|-----------------|
| Scry 1 after mulliganing | Requires a new UI interaction (look at top card, choose top or bottom). Adds disproportionate complexity for a rarely game-changing decision. Defer to a follow-up spec. |
| Smart bot mulligan logic | The bot always keeps. Evaluating opening hands (land count heuristics, curve analysis) is a separate AI task. |
| Animation for mulligan transitions | Card shuffle / fly-away animations are deferred to the U5 animation pass. |
| London Mulligan variant | London Mulligan (draw 7 every time, put back N at end) is more complex to track because the put-back count and hand size diverge. Vancouver is simpler and is the current MTG Standard rule. Can be switched later. |
| Partial Paris (free first mulligan) | Some casual formats give the first mulligan for free. Not implemented; all mulligans cost one card in this spec. |
| Hand size modification (e.g. Leyline of Abundance) | Starting hand size other than 7 is not supported. Cards that modify starting hand size are out of scope for MVP. |
| P1 vs P1 hotseat mulligan | The game is single-player P1 vs bot. Both players never simultaneously need to show their hands. Hotseat is superseded by G1 (bot). |
