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
- [x] The creature stays on your battlefield regardless of damage taken.
- [x] Hover panel shows "Indestructible" in oracle text.

---

### K4 — Hexproof

**Card**: Oakshield Troll (3/3 Creature — Troll, {1}{G})

**Steps**:
1. Play 2 Forests and cast Oakshield Troll.
2. Wait for P2's bot to cast Lightning Strike.

**Verification**:
- [x] Lightning Strike never lands on Oakshield Troll.
- [x] Hover panel shows "Hexproof" in oracle text.

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
- [x] Frozen Sentinel stays rotated (tapped) after P2's untap step.
- [x] Other P2 permanents untap normally.
- [x] Hover shows "Frozen Sentinel doesn't untap during your untap step."

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
- [x] Ironbark Wall is never highlighted as a possible attacker.
- [x] Ironbark Wall can block opponent creatures.
- [x] Hover shows "This creature can't attack."

---

### K10 — MustAttack

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Watch P2's bot play Reckless Berserker.
2. Observe combat when P2 has it on the battlefield.

**Verification**:
- [x] Reckless Berserker attacks every turn it is untapped.
- [x] Hover shows "This creature attacks each combat if able."

---

### K2 — Menace

**Card**: Reckless Berserker (2/1 Creature — Berserker, {R}) — in the Red deck (P2/bot)

**Steps**:
1. Allow Reckless Berserker to attack.
2. Attempt to block it with only one creature.

**Verification**:
- [x] Single blocker does not prevent damage to player.
- [x] Two blockers successfully block Reckless Berserker.
- [x] Hover shows "Menace" in the abilities.

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
- [x] First Thalia enters and stays on battlefield.
- [x] Second Thalia causes one to go to graveyard (only 1 Thalia remains).
- [x] Hover shows "Legendary" somewhere in the card info.

---

### R5 / R15 — Mana-cost activated abilities / Multiple abilities

**Card**: Sol Ring (Artifact, {1}) — in both decks

**Steps**:
1. Cast Sol Ring (costs {1}).
2. Tap it for mana: click Sol Ring on battlefield.

**Verification**:
- [x] Sol Ring taps and adds {C}{C} to mana pool.
- [x] HUD mana pool reflects the added mana.
- [x] Can use Sol Ring mana to cast creatures or spells.

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
- [x] Stack panel shows the triggered ability after Wild Bounty enters.
- [x] Hand size increases by 1 after the trigger resolves.
- [x] There is a priority window between the trigger going on stack and resolving.

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

### LS1 — Layer System (CR 613)

**Deck**: `TEST_DECK=ls1 cargo run -p echomancy-bevy`

**Cards**: Turn to Frog ({1}{U}), Twisted Image ({U}), Titanic Growth ({1}{G}), Giant Growth ({G}), Bear (2/2), Ironbark Wall (0/4), Ancient Guardian (4/5 Indestructible)

#### LS1.1 — Turn to Frog sets base P/T (Layer 7b)

**Steps**:
1. Play Islands + Forests over several turns.
2. Cast a Bear (2/2).
3. Cast Turn to Frog targeting the Bear.

**Verification**:
- [x] Bear becomes 1/1 after Turn to Frog resolves (hover to confirm P/T).
- [x] Bear's oracle text no longer shows any abilities.
- [x] Bear returns to 2/2 at end of turn.

#### LS1.2 — Turn to Frog + Giant Growth (Layer 7b before 7c)

**Steps**:
1. Cast Turn to Frog on a Bear (becomes 1/1).
2. Then cast Giant Growth on the same Bear (+3/+3).

**Verification**:
- [x] Bear shows 4/4 (1+3 / 1+3), NOT 5/5 — proves Layer 7b (set) applies before Layer 7c (modify).
- [x] Bear returns to 2/2 at end of turn (both effects expire).

#### LS1.3 — Twisted Image switches P/T (Layer 7d)

**Steps**:
1. Cast Ironbark Wall (0/4).
2. Cast Twisted Image targeting the Wall.

**Verification**:
- [x] Ironbark Wall becomes 4/0 after Twisted Image resolves.
- [x] Ironbark Wall dies immediately (SBA: 0 toughness).
- [x] You draw a card from Twisted Image's draw effect.

#### LS1.4 — Turn to Frog removes Indestructible (Layer 6)

**Steps**:
1. Cast Ancient Guardian (4/5 Indestructible).
2. Cast Turn to Frog targeting Ancient Guardian (becomes 1/1, loses all abilities).
3. Let the bot attack or shoot it with Lightning Strike.

**Verification**:
- [x] Ancient Guardian shows 1/1 after Turn to Frog.
- [x] Ancient Guardian no longer shows "Indestructible" in hover.
- [x] Ancient Guardian can now die to damage (no longer Indestructible).

#### LS1.5 — Titanic Growth (Layer 7c pump)

**Steps**:
1. Cast a Bear (2/2).
2. Cast Titanic Growth targeting the Bear (+4/+4).

**Verification**:
- [x] Bear shows 6/6 after Titanic Growth resolves.
- [x] Bear returns to 2/2 at end of turn.

#### LS1.6 — Stacked effects: Titanic Growth + Twisted Image

**Steps**:
1. Cast Ironbark Wall (0/4).
2. Cast Titanic Growth on the Wall (+4/+4 → 4/8).
3. Cast Twisted Image on the Wall (switch → 8/4).

**Verification**:
- [x] Ironbark Wall shows 8/4 after both spells resolve.
- [x] Wall survives (toughness > 0).

---

### R11 — Replacement Effects (CR 614)

**Deck**: `TEST_DECK=r11 cargo run -p echomancy-bevy`

**Cards**: Mending Light ({W} prevent 3), Trollhide ({G} regenerate), Bear (2/2), Ironbark Wall (0/4), Twisted Image ({U}), Turn to Frog ({1}{U}), Giant Growth ({G})

#### R11.1 — Prevention shield absorbs all damage

**Prepared hand**: Plains, Plains, Forest, Bear, Mending Light, Giant Growth, Sol Ring

**Steps**:
1. Turn 1: Play Plains.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Turn 3: Play Plains, cast Mending Light targeting the Bear (prevention shield of 3 active).
4. Wait for bot's Lightning Strike targeting the Bear (3 damage).

**Verification**:
- [ ] Bear survives Lightning Strike (prevention shield absorbs all 3 damage).
- [ ] Bear shows 2/2 with no damage markers after the strike.

#### R11.2 — Regeneration saves creature from lethal combat damage

**Prepared hand**: Forest, Forest, Plains, Bear, Trollhide, Giant Growth, Sol Ring

**Steps**:
1. Turn 1: Play Forest.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Turn 3: Play Plains.
4. During bot's combat, block a 3/3 Frozen Sentinel with the Bear.
5. Before combat damage, cast Trollhide ({G}) targeting the Bear (regeneration shield active).
6. Let combat damage resolve (3 damage to a 2/2 = lethal).

**Verification**:
- [ ] Bear survives lethal combat damage (regeneration replaces destroy).
- [ ] Bear is now tapped after regeneration.
- [ ] Bear has no damage marked (regeneration removes all damage).

#### R11.3 — Zero toughness bypasses regeneration (CR 704.5f)

**Prepared hand**: Forest, Island, Island, Ironbark Wall, Trollhide, Twisted Image, Sol Ring

**Steps**:
1. Turn 1: Play Forest.
2. Turn 2: Play Island, cast Ironbark Wall (0/4).
3. Turn 3: Play Island, cast Trollhide targeting Ironbark Wall (regen shield active).
4. Cast Twisted Image targeting Ironbark Wall (switch P/T → 4/0).

**Verification**:
- [ ] Ironbark Wall dies despite having a regeneration shield.
- [ ] Zero toughness is "put into graveyard", not "destroy" — regen doesn't apply.
- [ ] You draw a card from Twisted Image.

#### R11.4 — Prevention shield partially absorbs damage

**Prepared hand**: Plains, Forest, Forest, Bear, Mending Light, Giant Growth, Giant Growth

**Steps**:
1. Turn 1: Play Plains.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Turn 3: Play Forest, cast Mending Light on Bear (prevent 3).
4. Cast two Giant Growth on a bot creature to pump it high (or wait for enough combat damage > 3).
5. Block the pumped creature with Bear — total damage should exceed 3.

**Verification**:
- [ ] Prevention absorbs 3 of the incoming damage, the rest goes through.
- [ ] If remaining damage is lethal (≥2 for a 2/2 Bear), Bear dies.
- [ ] If remaining damage is <2, Bear survives with damage marked.

#### R11.5 — Turn to Frog does NOT interact with replacement effects (control test)

**Prepared hand**: Island, Island, Forest, Bear, Turn to Frog, Trollhide, Sol Ring

**Steps**:
1. Turn 1: Play Island.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Turn 3: Play Island, cast Trollhide on Bear (regen shield active).
4. Cast Turn to Frog on Bear (becomes 1/1, loses all abilities).
5. Wait for bot's Lightning Strike on Bear (3 damage to a 1/1).

**Verification**:
- [ ] Regeneration shield was registered before Turn to Frog → shield still exists in the replacement registry.
- [ ] Bear (now 1/1) takes 3 lethal damage → regeneration fires → Bear survives, tapped, no damage.
- [ ] Turn to Frog removes abilities from the creature, but does NOT remove replacement effects from the registry (they are separate from the layer system).

---

### R12 — Prevention Effects (CR 615)

**Deck**: `TEST_DECK=r12 cargo run -p echomancy-bevy`

**Cards**: Fog ({G} no target), Guardian Shield ({1}{W} target creature), Mending Light ({W} prevent 3), Bear (2/2), Ironbark Wall (0/4), Giant Growth ({G}), Trollhide ({G})

#### R12.1 — Fog prevents all combat damage

**Prepared hand**: Forest, Forest, Forest, Bear, Bear, Fog, Sol Ring

**Steps**:
1. Turn 1: Play Forest.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Turn 3: Play Forest, cast second Bear.
4. Let bot attack with creatures (Goblins, Frozen Sentinels).
5. Block with one Bear. Before combat damage, cast Fog ({G}).

**Verification**:
- [ ] After combat damage step, your blocking Bear has 0 damage (Fog prevented combat damage).
- [ ] Your life total did not decrease from unblocked attackers (Fog prevented that too).
- [ ] Bot's attacking creatures also took 0 damage from blockers (Fog prevents ALL combat damage).

#### R12.2 — Fog does NOT prevent spell damage

**Prepared hand**: Forest, Forest, Plains, Bear, Fog, Mending Light, Sol Ring

**Steps**:
1. Turn 1: Play Forest.
2. Turn 2: Play Forest, cast Bear (2/2).
3. Cast Fog at some point during the turn (it lasts until end of turn).
4. Wait for bot's Lightning Strike targeting the Bear.

**Verification**:
- [ ] Lightning Strike deals full 3 damage to the Bear (Fog does NOT prevent spell damage).
- [ ] Bear dies to lethal damage (3 ≥ 2 toughness).

#### R12.3 — Guardian Shield prevents all damage to target creature

**Prepared hand**: Plains, Plains, Forest, Bear, Guardian Shield, Giant Growth, Sol Ring

**Steps**:
1. Turn 1: Play Plains.
2. Turn 2: Play Plains, cast Sol Ring.
3. Turn 3: Play Forest, cast Bear (2/2).
4. Cast Guardian Shield ({1}{W}) targeting the Bear.
5. Block a bot attacker with the Bear.
6. After combat, wait for bot's Lightning Strike on the Bear.

**Verification**:
- [ ] Bear survives combat damage (Guardian Shield prevents it).
- [ ] Bear survives Lightning Strike (Guardian Shield prevents ALL damage, not just combat).
- [ ] Bear shows 2/2 with no damage marked throughout the turn.

#### R12.4 — Guardian Shield only protects the target

**Prepared hand**: Plains, Plains, Forest, Bear, Bear, Guardian Shield, Sol Ring

**Steps**:
1. Deploy two Bears.
2. Cast Guardian Shield on Bear A.
3. Wait for bot's Lightning Strike on Bear B (the unprotected one).

**Verification**:
- [ ] Bear B takes full 3 damage and dies (Guardian Shield does NOT protect it).
- [ ] Bear A remains undamaged.

#### R12.5 — Fog + Mending Light combo (multiple prevention effects)

**Prepared hand**: Forest, Forest, Plains, Bear, Fog, Mending Light, Sol Ring

**Steps**:
1. Deploy Bear.
2. Cast Mending Light on Bear (prevent 3 shield).
3. Cast Fog.
4. Let bot attack — block with Bear.

**Verification**:
- [ ] Bear takes 0 combat damage (Fog prevents it before Mending Light is needed).
- [ ] After combat, Mending Light shield is still active (wasn't consumed because Fog handled it).
- [ ] If bot casts Lightning Strike post-combat, Mending Light absorbs the 3 spell damage.

---

### CK1 — Regenerate Keyword (CR 701.15)

**Deck**: `TEST_DECK=ck1 cargo run -p echomancy-bevy`

**Cards**: River Troll (2/3 Troll, {2}{G}, "{G}: Regenerate"), Giant Growth ({G}), Trollhide ({G}), Turn to Frog ({1}{U}), Twisted Image ({U}), Ironbark Wall (0/4)

#### CK1.1 — Regenerate saves creature from lethal combat damage

**Prepared hand**: Forest, Forest, Forest, Forest, River Troll, Giant Growth, Sol Ring

**Steps**:
1. Turn 1: Play Forest.
2. Turn 2: Play Forest.
3. Turn 3: Play Forest, cast River Troll ({2}{G}).
4. Turn 4: Play Forest. When bot attacks with a creature with power >= 3, block with River Troll.
5. Before combat damage, tap a Forest and activate River Troll's regenerate ability ({G}).
6. Let combat damage resolve (lethal to a 2/3 Troll).

**Verification**:
- [ ] River Troll survives lethal combat damage.
- [ ] River Troll is now tapped after regeneration.
- [ ] River Troll has no damage marked.

#### CK1.2 — Regenerate saves from Lightning Strike

**Prepared hand**: Forest, Forest, Forest, Forest, River Troll, Giant Growth, Sol Ring

**Steps**:
1. Deploy River Troll (2/3).
2. Tap a Forest to activate regenerate ({G}) — shield active.
3. Wait for bot's Lightning Strike on River Troll (3 damage = lethal for 2/3).

**Verification**:
- [ ] River Troll survives Lightning Strike (regen shield intercepts the destroy from SBA).
- [ ] River Troll is tapped, no damage marked.

#### CK1.3 — Regeneration shield is consumed (single use)

**Prepared hand**: Forest, Forest, Forest, Forest, River Troll, River Troll, Sol Ring

**Steps**:
1. Deploy River Troll. Activate regenerate once ({G}).
2. Let the first lethal hit land (regen fires, Troll survives).
3. Let a second lethal hit land without re-activating regenerate.

**Verification**:
- [ ] First lethal hit: Troll survives (regen fires).
- [ ] Second lethal hit: Troll dies (no shield left).

#### CK1.4 — Turn to Frog removes regenerate ability but not existing shield

**Prepared hand**: Forest, Forest, Island, Island, River Troll, Turn to Frog, Sol Ring

**Steps**:
1. Deploy River Troll. Activate regenerate ({G}) — shield active.
2. Cast Turn to Frog on River Troll ({1}{U}) — becomes 1/1, loses all abilities.
3. Wait for bot's Lightning Strike (3 damage to a 1/1).

**Verification**:
- [ ] River Troll loses the regenerate ability (can't activate it again).
- [ ] The existing regeneration shield still fires (it's in the replacement registry, separate from abilities).
- [ ] River Troll (1/1) survives the 3 damage, tapped, no damage.

#### CK1.5 — Multiple activations stack shields

**Prepared hand**: Forest, Forest, Forest, Forest, Forest, River Troll, Sol Ring

**Steps**:
1. Deploy River Troll. Have 2+ Forests untapped.
2. Activate regenerate twice ({G} + {G}) — two shields active.
3. Take first lethal hit → first shield fires, Troll survives.
4. Take second lethal hit → second shield fires, Troll survives again.

**Verification**:
- [ ] River Troll survives two consecutive lethal damage events.
- [ ] After both shields consumed, a third lethal hit kills it.

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

### Layer System deck (P1, TEST_DECK=ls1) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 14 | Island | Blue mana |
| 10 | Forest | Green mana |
| 4 | Sol Ring | Mana acceleration |
| 8 | Bear (2/2) | Target bàsic |
| 4 | Ironbark Wall (0/4) | Target per Twisted Image (switch → 4/0 → mor) |
| 4 | Ancient Guardian (4/5 Indestructible) | Target per Turn to Frog (perd Indestructible) |
| 4 | Turn to Frog ({1}{U}) | LS1: Layer 6 (RemoveAllAbilities) + Layer 7b (SetPT 1/1) |
| 4 | Twisted Image ({U}) | LS1: Layer 7d (SwitchPT) + draw |
| 4 | Titanic Growth ({1}{G}) | LS1: Layer 7c (+4/+4) |
| 4 | Giant Growth ({G}) | LS1: Layer 7c (+3/+3) |

### Replacement Effects deck (P1, TEST_DECK=r11) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 10 | Plains | White mana (Mending Light) |
| 10 | Forest | Green mana (Trollhide) |
| 4 | Island | Blue mana (Turn to Frog, Twisted Image) |
| 4 | Sol Ring | Mana acceleration |
| 8 | Bear (2/2) | Target bàsic per prevention/regen |
| 4 | Ironbark Wall (0/4) | Target per zero-toughness edge case |
| 4 | Mending Light ({W}) | R11: Prevention shield (prevent next 3 damage) |
| 4 | Trollhide ({G}) | R11: Regeneration shield |
| 4 | Giant Growth ({G}) | Pump per edge cases |
| 4 | Twisted Image ({U}) | Switch P/T (zero-toughness test) |
| 4 | Turn to Frog ({1}{U}) | Control: removes abilities, test interaction |

### Prevention Effects deck (P1, TEST_DECK=r12) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 10 | Plains | White mana (Guardian Shield, Mending Light) |
| 10 | Forest | Green mana (Fog, Trollhide, Giant Growth) |
| 4 | Sol Ring | Mana acceleration |
| 8 | Bear (2/2) | Target bàsic per prevention |
| 4 | Ironbark Wall (0/4) | Edge case target |
| 4 | Fog ({G}) | R12: Global combat damage prevention |
| 4 | Guardian Shield ({1}{W}) | R12: Full-turn prevention on target creature |
| 4 | Mending Light ({W}) | R11: Depleting prevention shield (combo testing) |
| 4 | Giant Growth ({G}) | Pump spell |
| 4 | Trollhide ({G}) | R11: Regeneration shield |
| 4 | Twisted Image ({U}) | Edge case testing |

### Regenerate deck (P1, TEST_DECK=ck1) — 60 cards

| Count | Card | Feature |
|-------|------|---------|
| 12 | Forest | Green mana (Troll + regenerate activation) |
| 6 | Plains | White mana (Mending Light) |
| 4 | Island | Blue mana (Turn to Frog, Twisted Image) |
| 4 | Sol Ring | Mana acceleration |
| 8 | River Troll (2/3, {G}: Regenerate) | CK1: Regenerate keyword showcase |
| 4 | Mending Light ({W}) | Prevention combo testing |
| 4 | Giant Growth ({G}) | Pump spell |
| 4 | Trollhide ({G}) | R11: Regen instant (comparison) |
| 4 | Turn to Frog ({1}{U}) | Test: removes ability but not existing shield |
| 2 | Ironbark Wall (0/4) | Zero-toughness edge case |
| 4 | Twisted Image ({U}) | Zero-toughness test |
| 4 | Bear (2/2) | Basic targets |

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
