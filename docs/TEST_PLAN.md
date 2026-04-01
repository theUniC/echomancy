# Manual Test Plan

This document describes how to manually verify each feature implemented in the
Echomancy engine. Run the game with `cargo run -p echomancy-bevy` before testing.

Both prebuilt decks now contain showcase cards that exercise each feature.
P1 plays the Green deck; P2 (bot) plays the Red deck.

---

## How to use this test plan

1. Start the game: `cargo run -p echomancy-bevy`
2. Complete the mulligan for P1 (keep or redraw).
3. Play through turns until you can get the relevant card onto the battlefield.
4. Verify the expected behavior described in each section.

Hover over any card to see its oracle text and mana cost in the detail panel.

---

## Feature Tests

### K3 — Indestructible

**Card**: Ancient Guardian (4/5 Creature — Elemental, {2}{G})

**Steps**:
1. Play 3 Forests over 3 turns to accumulate mana.
2. Cast Ancient Guardian from hand.
3. Let the bot (P2) attack with Goblins or shoot it with Lightning Strike.

**Expected behavior**:
- Ancient Guardian survives Lightning Strike (3 damage does not destroy it).
- Ancient Guardian survives lethal combat damage without being destroyed.
- It remains on the battlefield after any "destroy" effect.

**Verification**: The creature stays on your battlefield regardless of damage taken.

---

### K4 — Hexproof

**Card**: Oakshield Troll (3/3 Creature — Troll, {1}{G})

**Steps**:
1. Play 2 Forests and cast Oakshield Troll.
2. Wait for P2's bot to cast Lightning Strike.
3. Observe the targeting phase.

**Expected behavior**:
- P2's Lightning Strike cannot target Oakshield Troll.
- The spell is forced to target another legal target (a different creature or P1 directly).
- If no other target is available, Lightning Strike cannot be cast.

**Verification**: Lightning Strike never lands on Oakshield Troll. The hover panel
shows "Hexproof" in the oracle text.

---

### K5 — Shroud (unit tested only)

Shroud is implemented in the targeting specification alongside Hexproof.
It is not currently in either prebuilt deck. Verified by unit tests in
`can_cast_spell.rs` and `can_declare_attacker.rs`.

---

### K7 — DoesNotUntap

**Card**: Frozen Sentinel (3/3 Creature — Golem, {1}{R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Frozen Sentinel.
2. Observe the untap step at the start of P2's next turn.

**Expected behavior**:
- All other P2 permanents untap normally.
- Frozen Sentinel remains tapped after P2's untap step.
- It stays tapped for the rest of the game (it can only act the turn it enters,
  because it has Haste but never untaps).

**Verification**: The Frozen Sentinel card stays rotated (tapped) in P2's battlefield
even after the turn changes. Hover to see "Frozen Sentinel doesn't untap during your
untap step."

---

### K8 — EntersTapped

**Card**: Thornwood Tapland (Land, {T}: Add {G}) — in the Green deck (P1)

**Steps**:
1. Play Thornwood Tapland from hand during your main phase.

**Expected behavior**:
- The land enters the battlefield already tapped (rotated).
- You cannot immediately tap it for mana on the same turn it was played.
- On your next turn it untaps normally and can produce {G}.

**Verification**: The land card appears rotated immediately after playing it.
Attempting to activate its mana ability while tapped should fail or produce no mana.

---

### K9 — CannotAttack

**Card**: Ironbark Wall (0/4 Creature — Plant Wall, {G}) — in the Green deck (P1)

**Steps**:
1. Play 1 Forest and cast Ironbark Wall.
2. On your next turn, try to declare Ironbark Wall as an attacker.

**Expected behavior**:
- Ironbark Wall cannot be selected as an attacker during the Declare Attackers step.
- The "Declare Attackers" UI does not offer Ironbark Wall as an attackable creature.
- It can still block normally (CannotAttack does not imply CannotBlock).

**Verification**: Ironbark Wall is never highlighted as a possible attacker. Hover to
see "This creature can't attack."

---

### K10 — MustAttack

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Reckless Berserker.
2. Observe combat when P2 has Reckless Berserker on the battlefield and can attack.

**Expected behavior**:
- Reckless Berserker attacks every combat if able.
- The bot will always declare it as an attacker when it is untapped and not
  prevented from attacking by other rules.

**Verification**: Reckless Berserker appears in the Declare Attackers list every turn.
Hover to see "This creature attacks each combat if able."

---

### K2 — Menace

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Allow Reckless Berserker to attack.
2. Attempt to block it with only one creature.

**Expected behavior**:
- Blocking Reckless Berserker with only one creature is illegal.
- You must assign at least two blockers to block it, or let it through unblocked.

**Verification**: A single-creature block attempt is rejected. The game requires two
or more blockers to legally block Reckless Berserker.

---

### R2 — Subtypes

**Card**: Bear (2/2 Creature — Bear, {1}{G}), Goblin (1/1 Creature — Goblin, {R}),
Oakshield Troll (3/3 Creature — Troll), etc.

**Steps**:
1. Hover over any creature on the battlefield or in hand.

**Expected behavior**:
- The card detail panel shows the full type line, e.g. "Creature — Bear".
- Multi-subtype cards show all subtypes, e.g. "Creature — Human Druid" for
  Thalia, Forest Keeper.

**Verification**: Type line is visible in the hover panel with the dash separator
before the subtype.

---

### R3 — Legendary

**Card**: Thalia, Forest Keeper (2/2 Legendary Creature — Human Druid, {G})
— in the Green deck (P1)

**Steps**:
1. Cast Thalia, Forest Keeper.
2. Cast a second copy of Thalia, Forest Keeper (if drawn again).

**Expected behavior**:
- First Thalia enters the battlefield normally.
- When the second Thalia attempts to enter, the Legendary rule (CR 704.5j) triggers:
  both copies are moved to the graveyard as a state-based action.

**Verification**: Playing a second Thalia causes both copies to immediately die.
The hover panel shows "Legendary" in the oracle text.

---

### R5 / R15 — Mana-cost activated abilities / Multiple abilities

**Card**: Sol Ring (Artifact, {1}) — in both decks

**Steps**:
1. Cast Sol Ring.
2. Tap it for mana: click Sol Ring, select the {T}: Add {C}{C} ability.

**Expected behavior**:
- Tapping Sol Ring adds 2 colorless mana to your mana pool.
- The mana pool counter in the HUD increases by 2.

**Verification**: HUD shows +2 colorless mana after activation. Cards that cost 2
generic mana become castable.

---

### R9 — Tokens (unit tested only)

Token generation via CLIPS rules is implemented in the rules engine but no card
in the current catalog creates tokens. This feature is verified by unit tests
in the CLIPS integration layer. Manual testing requires a token-generating card
(not yet in the catalog).

---

### R10 — Triggered abilities on the stack

**Card**: Wild Bounty (Enchantment, {1}{G}) — in both decks

**Steps**:
1. Cast Wild Bounty.
2. Observe the stack during resolution.

**Expected behavior**:
- When Wild Bounty resolves and enters the battlefield, an ETB triggered ability
  appears on the stack: "Wild Bounty ETB — draw a card".
- After the triggered ability resolves, P1 draws a card (hand size increases by 1).

**Verification**: Stack panel briefly shows the triggered ability. After it resolves,
P1's hand increases by 1 card. This verifies CR 603.3 triggered ability handling.

---

### MM1 — Scry (unit tested only)

Scry is implemented as a `RulesAction` in the CLIPS rules engine. No card in the
current catalog produces a Scry effect. Verified by unit tests in
`clips/mod.rs` and `rules_engine.rs`.

---

### MM2 — Mill (unit tested only)

Mill (move cards from library to graveyard) is implemented as a `RulesAction`.
No card in the current catalog produces a Mill effect. Verified by unit tests.

---

### MM3 — Discard (unit tested only)

Discard is implemented as a `RulesAction`. No card in the current catalog
requires discarding. Verified by unit tests.

---

### C5b — SBA: draw from empty library (unit tested only)

State-Based Action: a player who must draw from an empty library loses the game.
This edge case is not easily reproducible manually in a normal game. Verified by
unit tests in `domain/game/sba.rs`.

---

## Current deck compositions

### Green deck (P1) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 16 | Forest | Basic land |
| 2 | Thornwood Tapland | K8 EntersTapped |
| 16 | Bear (2/2) | Vanilla creature |
| 2 | Oakshield Troll (3/3 Hexproof) | K4 Hexproof |
| 2 | Ancient Guardian (4/5 Indestructible) | K3 Indestructible |
| 2 | Ironbark Wall (0/4 CannotAttack) | K9 CannotAttack |
| 2 | Thalia, Forest Keeper (2/2 Legendary First Strike) | R3 Legendary |
| 12 | Giant Growth (instant +3/+3) | Combat trick |
| 4 | Sol Ring (artifact) | R5/R15 Mana ability |
| 2 | Wild Bounty (enchantment ETB draw) | R10 Triggered ability |

### Red deck (P2/bot) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 18 | Mountain | Basic land |
| 16 | Goblin (1/1) | Vanilla creature |
| 2 | Reckless Berserker (2/1 Menace MustAttack) | K2 Menace, K10 MustAttack |
| 4 | Frozen Sentinel (3/3 Haste DoesNotUntap) | K7 DoesNotUntap |
| 14 | Lightning Strike (instant 3 damage) | Direct damage |
| 4 | Sol Ring (artifact) | R5/R15 Mana ability |
| 2 | Wild Bounty (enchantment ETB draw) | R10 Triggered ability |
