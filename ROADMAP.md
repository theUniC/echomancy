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
- Clearly separate `cost` from `effect`

**Examples**
- Tap this creature
- Sacrifice a permanent
- Pay mana

**Notes**
- No UI required
- Domain-level tests only

---

### 2️⃣ Power / Toughness + Counters
**Goal**
- Model base power and toughness
- Support +1/+1 counters

**Scope**
- Simple counters
- No continuous effects yet

---

### 3️⃣ Static Abilities — Local (MVP)
**Goal**
- Support static abilities that affect only the permanent itself

**Examples**
- Haste
- Enters tapped
- Cannot attack
- Vigilance (simplified)

**Notes**
- Evaluated as local state or computed flags
- No dependency resolution
- No layer system
- Required for a minimally honest Magic UI

---

### 4️⃣ Simple Continuous Effects (Lords)
**Goal**
- Support simple global modifiers

**Examples**
- “Other elves you control get +1/+1”

**Accepted limitations**
- No full layer system
- No dependency resolution
- No interaction with rule-changing effects

---

## 🔴 Explicitly Out of Core (MVP)

These features **do not block UI** and are intentionally excluded from the initial core:

- Full layer rules
- Complex replacement effects
- Fully generalized stack actions
- Spell copying
- Advanced alternative costs
- Fine-grained automatic priority passing
- Automatic parsing of card text
- Expert-system approaches (Arena / CLIPS / GRP)

---

## 🔵 Long-Term Correctness Goals (Post-MVP)

These items are **not required to close the Core**, but are part of
Echomancy’s long-term vision as a correct, transparent Magic rules engine.

They are documented to:
- Avoid architectural dead-ends
- Make limitations explicit
- Set expectations for contributors and players

---

### 7-Layer Continuous Effects System

**Goal**
- Support Magic’s official continuous effects ordering model

**Scope (Future)**
- Copy effects
- Control-changing effects
- Text-changing effects
- Type-changing effects
- Color-changing effects
- Ability adding/removing effects
- Power/Toughness layers (including sublayers)

**Notes**
- Requires dependency resolution and deterministic evaluation
- Explicitly deferred
- No partial or simplified implementation in the MVP

---

### Advanced Static & Rule-Changing Abilities

**Examples**
- “Creatures lose all abilities”
- “Players can’t cast spells during combat”
- Replacement effects (“If X would happen, instead Y”)

**Status**
- Depends on the 7-layer system
- Explicitly postponed

---

### Card Text Interpretation / Rule Engines

**Examples**
- Interpreter patterns
- Expert systems (e.g. CLIPS / GRP-style approaches)
- Automatic Oracle text parsing

**Status**
- Research topic
- Not required for Echomancy to be playable, fair, or transparent

---

## 🧩 What Unlocks UI Work

Once the following are completed:
- ~~Mana Pool MVP~~ ✅
- Costs
- Power/Toughness + Counters
- Static Abilities — Local (MVP)

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
