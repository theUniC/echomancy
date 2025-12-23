# Echomancy — Core Rules Roadmap (Living Document)

This document defines the scope of the **core rules engine** of Echomancy.
It is a living document and will be updated as milestones are completed.

The final goal of the core is:
👉 to provide a stable, well-tested rules engine  
👉 so that a minimal UI can be built without reworking fundamentals

---

## 🎯 Definition of “Core Closed”

The **core is considered closed** when:

- A full turn cycle works end to end without critical stubs
- Basic Magic interactions are modeled correctly
- Simple real decks (e.g. Elves) can be implemented and tested
- No major engine refactor is required to start building a UI

The core **does NOT aim to cover all of Magic**.

---

## 🟢 Current State (Implemented & Validated)

### Turns and Phases
- Full turn structure
- Normal phases and extra phases
- Correct resume of normal flow after extra phases
- Tests covering real scenarios

### Priority and Stack
- Priority alternation
- Stack resolution
- Spells and abilities on the stack
- CI green with passing tests

### Cards and Zones
- Hand / Battlefield / Graveyard
- Play land (no stack)
- Cast spell (uses stack)
- Correct zone transitions

### ETB (Enter the Battlefield)
- ETB triggered when permanents resolve from the stack
- Known limitation: ETB targeting is simplified (documented)

### Creatures (MVP)
- State tracking: tapped / attacking / attacked-this-turn
- Declare attackers
- Basic attack restrictions
- Correct reset on phase and turn changes
- Extensive tests (including extra combat phases)

### Abilities
- Activated abilities
- Triggered abilities
- Integrated with the stack
- Real tests (Elves)

### Mana Pool (MVP)
- Mana pool per player (6 colors: W, U, B, R, G, C)
- Add and spend mana operations
- Pool clearing at CLEANUP step (MVP behavior)
- Error handling (insufficient mana, invalid amounts)
- Complete test coverage
- Known limitation: pools clear only at CLEANUP, not per-step (documented)

---

## 🟡 Pending to Close the Core

### 1️⃣ Costs (Beyond Mana)
**Goal**
- Introduce a first-class `Cost` concept
- Clearly separate `cost` from `effect`

**Examples**
- Tap this permanent
- Sacrifice a permanent
- Pay X mana
- Composite costs (e.g. tap + mana)

**Notes**
- No UI required
- Domain-level tests only
- Costs must be reusable by:
  - spells
  - activated abilities
  - future planeswalker abilities

---

### 2️⃣ Permanent Types — MVP
**Goal**
- Support all core permanent types required for a minimal UI

**Permanent Types**
- Creature (already implemented)
- Land (already implemented)
- Artifact
- Enchantment
- Planeswalker

**Scope**
- Correct battlefield presence
- Zone transitions
- Minimal state per type
- Targetable as permanents

**Explicitly Out**
- Auras (attachment rules deferred)
- Equipment (attachment rules deferred)
- Damage rules specific to planeswalkers (handled later with combat)

---

### 3️⃣ Power / Toughness + Counters
**Goal**
- Model combat-relevant numeric state

**Scope**
- Base power / toughness
- +1/+1 counters
- Counter application and removal
- Lethal damage checks (no combat yet)

**Accepted limitations**
- No continuous effects yet
- No layer system

---

### 4️⃣ Combat — Resolution MVP
**Goal**
- Make combat real and resolvable

**Scope**
- Declare attackers
- Declare blockers
- Damage assignment
- Damage resolution
- Creature destruction
- Damage to players

**Notes**
- Uses Power/Toughness
- No advanced combat abilities yet (first strike, trample, etc.)

---

### 5️⃣ Static Abilities — MVP
**Goal**
- Support always-on effects that modify game state

**Examples**
- “Other elves you control get +1/+1”
- Simple global modifiers

**Scope**
- Static effects evaluated dynamically
- Limited to battlefield interactions

**Accepted limitations**
- No full 7-layer system yet
- No dependency resolution between static effects

**Future**
- 7-layer system planned, but explicitly out of Core MVP

---

## 🔴 Explicitly Out of Core (for now)

These features **do not block UI** and are intentionally excluded from the initial core:

- Full 7-layer rules
- Complex replacement effects
- Spell copying
- Advanced alternative costs
- Full attachment rules (auras, equipment)
- Fine-grained automatic priority passing
- Automatic parsing of card text
- Expert-system approaches (Arena / CLIPS / GRP)

---

## 🧩 What Unlocks UI Work

Once the following are completed:
- ~~Mana Pool MVP~~ ✅
- Costs
- Permanent Types MVP
- Power/Toughness + Counters
- Combat MVP

We can safely start:
- Zone UI
- Stack UI
- Priority UI
- Combat UI
- Target selection UI

Without risk of reworking the engine.

---

## 🛠️ How This Document Is Maintained

- Each PR that closes a block:
  - Marks it as 🟢
  - References relevant tests
- Known limitations are explicitly documented
- Nothing is removed without discussion

This document is the **single source of truth for the roadmap**.
