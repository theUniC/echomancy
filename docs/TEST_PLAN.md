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

**Verification**:
- [ ] The creature stays on your battlefield regardless of damage taken.
- [ ] Hover panel shows "Indestructible" in oracle text.

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

**Verification**:
- [ ] Lightning Strike never lands on Oakshield Troll.
- [ ] Hover panel shows "Hexproof" in oracle text.

---

### K5 — Shroud (unit tested only)

- [x] Verified by unit tests (`cargo test -- shroud`)

Shroud is implemented alongside Hexproof but no card in the decks uses it.

---

### K7 — DoesNotUntap

**Card**: Frozen Sentinel (3/3 Creature — Golem, {1}{R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Frozen Sentinel.
2. Observe the untap step at the start of P2's next turn.

**Expected behavior**:
- All other P2 permanents untap normally.
- Frozen Sentinel remains tapped after P2's untap step.
- It stays tapped for the rest of the game (it has Haste so it attacks once, then stays tapped).

**Verification**:
- [ ] Frozen Sentinel stays rotated (tapped) after P2's untap step.
- [ ] Other P2 permanents untap normally.
- [ ] Hover shows "Frozen Sentinel doesn't untap during your untap step."

---

### K8 — EntersTapped

**Card**: Thornwood Tapland (Land, {T}: Add {G}) — in the Green deck (P1)

**Steps**:
1. Play Thornwood Tapland from hand during your main phase.

**Expected behavior**:
- The land enters the battlefield already tapped (rotated).
- You cannot tap it for mana on the same turn it was played.
- On your next turn it untaps normally and can produce {G}.

**Verification**:
- [x] Land appears rotated immediately after playing it.
- [x] Cannot activate mana ability the turn it enters.
- [x] Untaps and works normally next turn.

---

### K9 — CannotAttack

**Card**: Ironbark Wall (0/4 Creature — Plant Wall, {G}) — in the Green deck (P1)

**Steps**:
1. Play 1 Forest and cast Ironbark Wall.
2. On your next turn, try to declare Ironbark Wall as an attacker.

**Expected behavior**:
- Ironbark Wall cannot be selected as an attacker.
- It can still block normally.

**Verification**:
- [ ] Ironbark Wall is never highlighted as a possible attacker.
- [ ] Ironbark Wall can block opponent creatures.
- [ ] Hover shows "This creature can't attack."

---

### K10 — MustAttack

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Reckless Berserker.
2. Observe combat when P2 has it on the battlefield.

**Expected behavior**:
- Reckless Berserker attacks every combat if able.
- The bot always declares it as an attacker.

**Verification**:
- [ ] Reckless Berserker attacks every turn it is untapped.
- [ ] Hover shows "This creature attacks each combat if able."

---

### K2 — Menace

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Allow Reckless Berserker to attack.
2. Attempt to block it with only one creature.

**Expected behavior**:
- Blocking with only one creature is effectively ignored — the attacker hits you as if unblocked.
- You need at least two blockers to actually block it.

**Verification**:
- [ ] Single blocker does not prevent damage to player.
- [ ] Two blockers successfully block Reckless Berserker.
- [ ] Hover shows "Menace" in the abilities.

---

### R2 — Subtypes

**Cards**: Bear, Goblin, Oakshield Troll, Thalia, Ironbark Wall, etc.

**Steps**:
1. Hover over any creature on the battlefield or in hand.

**Expected behavior**:
- Type line shows subtypes with em-dash separator: "Creature — Bear", "Creature — Troll", etc.
- Lands show "Land — Forest", "Land — Mountain".

**Verification**:
- [x] Bear shows "Creature — Bear"
- [x] Goblin shows "Creature — Goblin"
- [x] Thalia shows "Creature — Human Druid"
- [x] Forest shows "Land — Forest"
- [x] Thornwood Tapland shows "Land — Forest"

---

### R3 — Legendary

**Card**: Thalia, Forest Keeper (2/2 Legendary Creature — Human Druid, {G})

**Steps**:
1. Cast Thalia, Forest Keeper.
2. If you draw a second copy, cast it too.

**Expected behavior**:
- First Thalia enters normally.
- When the second Thalia enters, the Legend rule (CR 704.5j) triggers as SBA: the duplicate goes to graveyard (oldest kept).

**Verification**:
- [ ] First Thalia enters and stays on battlefield.
- [ ] Second Thalia causes one to go to graveyard (only 1 Thalia remains).
- [ ] Hover shows "Legendary" somewhere in the card info.

---

### R5 / R15 — Mana-cost activated abilities / Multiple abilities

**Card**: Sol Ring (Artifact, {1}) — in both decks

**Steps**:
1. Cast Sol Ring (costs {1}).
2. Tap it for mana: click Sol Ring on battlefield.

**Expected behavior**:
- Tapping Sol Ring adds 2 colorless mana to your mana pool.
- The mana pool counter in the HUD increases.

**Verification**:
- [ ] Sol Ring taps and adds {C}{C} to mana pool.
- [ ] HUD mana pool reflects the added mana.
- [ ] Can use Sol Ring mana to cast creatures or spells.

---

### R9 — Tokens (unit tested only)

- [x] Verified by unit tests (`cargo test -- token`)

No card in the decks creates tokens. Feature verified at domain level.

---

### R10 — Triggered abilities on the stack

**Card**: Wild Bounty (Enchantment, {1}{G}) — in both decks

**Steps**:
1. Cast Wild Bounty.
2. Watch the stack as it resolves.

**Expected behavior**:
- When Wild Bounty enters the battlefield, an ETB triggered ability goes on the stack.
- After the triggered ability resolves, P1 draws a card.

**Verification**:
- [ ] Stack panel shows the triggered ability after Wild Bounty enters.
- [ ] Hand size increases by 1 after the trigger resolves.
- [ ] There is a priority window between the trigger going on stack and resolving.

---

### MM1 — Scry (unit tested only)

- [x] Verified by unit tests (`cargo test -- scry`)

---

### MM2 — Mill (unit tested only)

- [x] Verified by unit tests (`cargo test -- mill`)

---

### MM3 — Discard (unit tested only)

- [x] Verified by unit tests (`cargo test -- discard`)

---

### C5b — SBA infinite loop declares draw (unit tested only)

- [x] Verified by unit tests (`cargo test -- sba`)

---

## Quick unit test verification

Run all unit tests for features without showcase cards:

```bash
cargo test -- shroud scry mill discard sba token
```

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
