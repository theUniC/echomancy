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

**Verification**:
- [ ] The creature stays on your battlefield regardless of damage taken.
- [ ] Hover panel shows "Indestructible" in oracle text.

---

### K4 — Hexproof

**Card**: Oakshield Troll (3/3 Creature — Troll, {1}{G})

**Steps**:
1. Play 2 Forests and cast Oakshield Troll.
2. Wait for P2's bot to cast Lightning Strike.

**Verification**:
- [ ] Lightning Strike never lands on Oakshield Troll.
- [ ] Hover panel shows "Hexproof" in oracle text.

---

### K5 — Shroud

- [x] Verified by unit tests (`cargo test -- shroud`)

No card in the decks uses Shroud. Implemented alongside Hexproof.

---

### K7 — DoesNotUntap

**Card**: Frozen Sentinel (3/3 Creature — Golem, {1}{R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Frozen Sentinel.
2. Observe the untap step at the start of P2's next turn.

**Verification**:
- [ ] Frozen Sentinel stays rotated (tapped) after P2's untap step.
- [ ] Other P2 permanents untap normally.
- [ ] Hover shows "Frozen Sentinel doesn't untap during your untap step."

---

### K8 — EntersTapped

**Card**: Thornwood Tapland (Land, {T}: Add {G}) — in the Green deck (P1)

**Steps**:
1. Play Thornwood Tapland from hand during your main phase.

**Verification**:
- [x] Land appears rotated immediately after playing it.
- [x] Cannot activate mana ability the turn it enters.
- [x] Untaps and works normally next turn.

---

### K9 — CannotAttack / Defender

**Card**: Ironbark Wall (0/4 Creature — Plant Wall, {G}) — in the Green deck (P1)

**Steps**:
1. Play 1 Forest and cast Ironbark Wall.
2. On your next turn, try to declare Ironbark Wall as an attacker.

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

**Verification**:
- [ ] Reckless Berserker attacks every turn it is untapped.
- [ ] Hover shows "This creature attacks each combat if able."

---

### K2 — Menace

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Allow Reckless Berserker to attack.
2. Attempt to block it with only one creature.

**Verification**:
- [ ] Single blocker does not prevent damage to player.
- [ ] Two blockers successfully block Reckless Berserker.
- [ ] Hover shows "Menace" in the abilities.

---

### K11.1 — Defender

- [x] Verified by unit tests (`cargo test -- defender`)

Defender is the official keyword for CannotAttack. Ironbark Wall uses CannotAttack directly; both are checked in combat validation.

---

### K11.32 — Changeling

- [x] Verified by unit tests (`cargo test -- changeling`)

A creature with Changeling has all creature types. `has_subtype()` returns true for any query. No showcase card in decks.

---

### K11.33 — Devoid

- [x] Verified by unit tests (`cargo test -- devoid`)

A card with Devoid is colorless regardless of mana cost. `colors()` returns empty. No showcase card in decks.

---

### K11.5 — Fear

- [x] Verified by unit tests (`cargo test -- fear`)

Can't be blocked except by artifact creatures and/or black creatures. No showcase card in decks.

---

### K11.7 — Skulk

- [x] Verified by unit tests (`cargo test -- skulk`)

Can't be blocked by creatures with greater power. No showcase card in decks.

---

### K11.8 — Shadow

- [x] Verified by unit tests (`cargo test -- shadow`)

Can only block or be blocked by creatures with Shadow. No showcase card in decks.

---

### K11.9 — Horsemanship

- [x] Verified by unit tests (`cargo test -- horsemanship`)

Like Flying but only Horsemanship blocks Horsemanship. No showcase card in decks.

---

### R2 — Subtypes

**Cards**: Bear, Goblin, Oakshield Troll, Thalia, Ironbark Wall, etc.

**Steps**:
1. Hover over any creature on the battlefield or in hand.

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

**Verification**:
- [ ] Sol Ring taps and adds {C}{C} to mana pool.
- [ ] HUD mana pool reflects the added mana.
- [ ] Can use Sol Ring mana to cast creatures or spells.

---

### R9 — Tokens

- [x] Verified by unit tests (`cargo test -- token`)

No card in the decks creates tokens. Feature verified at domain level.

---

### R10 — Triggered abilities on the stack

**Card**: Wild Bounty (Enchantment, {1}{G}) — in both decks

**Steps**:
1. Cast Wild Bounty.
2. Watch the stack as it resolves.

**Verification**:
- [ ] Stack panel shows the triggered ability after Wild Bounty enters.
- [ ] Hand size increases by 1 after the trigger resolves.
- [ ] There is a priority window between the trigger going on stack and resolving.

---

### P10.1 — Poison counters

- [x] Verified by unit tests (`cargo test -- poison`)

Player with 10+ poison counters loses. No card in decks inflicts poison (needs Toxic/Infect).

---

### P10.2 — -1/-1 counters

- [x] Verified by unit tests (`cargo test -- minus_one`)

-1/-1 counters reduce P/T. Counter annihilation SBA removes +1/+1 and -1/-1 pairs.

---

### P10.6 — Counter annihilation SBA

- [x] Verified by unit tests (`cargo test -- counter_annihilation`)

When a permanent has both +1/+1 and -1/-1 counters, pairs are removed as SBA.

---

### P10.10 — Bounce (Return to Hand)

- [x] Verified by unit tests (`cargo test -- return_permanent_to_hand`)

`RulesAction::ReturnToHand` moves a permanent from battlefield to owner's hand.

---

### P15.1 — Surveil

- [x] Verified by unit tests (`cargo test -- surveil`)

Like Scry but cards go to graveyard instead of library bottom.

---

### MM1 — Scry

- [x] Verified by unit tests (`cargo test -- scry`)

---

### MM2 — Mill

- [x] Verified by unit tests (`cargo test -- mill`)

---

### MM3 — Discard

- [x] Verified by unit tests (`cargo test -- discard`)

---

### C5b — SBA infinite loop declares draw

- [x] Verified by unit tests (`cargo test -- sba`)

---

### P10.17 — Treasure tokens

- [x] Verified by unit tests (`cargo test -- treasure`)

`RulesAction::CreateTreasure` creates an Artifact — Treasure token on the battlefield.

---

### P10.15 — Clue tokens (Investigate)

- [x] Verified by unit tests (`cargo test -- clue`)

`RulesAction::Investigate` creates an Artifact — Clue token on the battlefield.

---

### P10.16 — Food tokens

- [x] Verified by unit tests (`cargo test -- food`)

`RulesAction::CreateFood` creates an Artifact — Food token on the battlefield.

---

### P10.12 — Fight mechanic

- [x] Verified by unit tests (`cargo test -- fight`)

Each creature deals damage equal to its power to the other. SBA checked after.

---

### P15.6 — Bolster N

- [x] Verified by unit tests (`cargo test -- bolster`)

Put N +1/+1 counters on the creature you control with the least toughness.

---

### P15.7 — Adapt N

- [x] Verified by unit tests (`cargo test -- adapt`)

If creature has no +1/+1 counters, put N. Otherwise no-op.

---

### K11.14 — Toxic N

- [x] Verified by unit tests (`cargo test -- toxic`)

When a creature with Toxic N deals combat damage to a player, that player gets N poison counters.

---

### P12.15 — Kindred card type

- [x] Verified by unit tests (`cargo test -- kindred`)

`CardType::Kindred` added. Kindred cards have creature subtypes without being creatures.

---

### MA5 — Snow supertype

- [x] Verified by unit tests (`cargo test -- snow`)

`is_snow` flag on CardDefinition. Snow mana payment logic not yet implemented.

---

## Quick unit test verification

Run all unit tests for features without showcase cards:

```bash
cargo test -- shroud changeling devoid fear skulk shadow horsemanship defender scry mill discard sba token poison surveil counter_annihilation return_permanent_to_hand treasure clue food fight bolster adapt toxic kindred snow
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
| 2 | Ironbark Wall (0/4 CannotAttack) | K9 CannotAttack/Defender |
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
