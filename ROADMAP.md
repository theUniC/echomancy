# Echomancy — Core Rules Roadmap (Living Document)

This document defines the scope of the **core rules engine** of Echomancy.
It is a living document and will be updated as milestones are completed.

Echomancy aims to be:

* 👉 an open, transparent, and fair Magic rules engine  
* 👉 focused on correctness, testability, and long-term maintainability  
* 👉 not constrained by commercial shortcuts or opaque rule systems

---

## 🎯 Definition of "Core Closed"

The **core is considered closed** when:

- A full turn cycle works end to end without critical stubs
- Core Magic interactions are modeled correctly
- Simple real decks (e.g. Elves) can be implemented and tested
- No major engine refactor is required to start building a minimal UI

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
- Mana pool per player (W, U, B, R, G, C)
- Add and spend mana operations
- Pool clearing at CLEANUP step (MVP behavior)
- Error handling and complete test coverage
- Known limitation: pools clear only at CLEANUP (documented)

### Costs (Beyond Mana)
- Explicit cost model
- Separation of cost and effect
- Atomic cost payment
- Supported costs: ManaCost, TapSelfCost, SacrificeSelfCost
- Reusable by spells and abilities
- Known limitations documented with TODOs

### Permanent Types (MVP)
- Creature, Land, Artifact, Enchantment, Planeswalker
- Correct battlefield behavior and zone transitions
- Multiple types per card supported
- Planeswalker placeholder state
- Known limitations:
  - No loyalty yet
  - No attachments (Auras / Equipment deferred)

### Power / Toughness + Counters (MVP)
- Base power and toughness for creatures
- +1/+1 counter support
- Counter addition and removal with validation
- Current power/toughness calculation
- Comprehensive test coverage
- Known limitations:
  - No static abilities applied yet
  - No layer system
  - No temporary modifiers
  - Only +1/+1 counters supported

### Combat Resolution (MVP)
- Declare attackers (validates untapped, not attacked this turn)
- Declare blockers (**1-to-1 blocking only**)
- Damage assignment (simultaneous)
- Damage resolution during COMBAT_DAMAGE step
- State-based actions for creature destruction
- Damage to players from unblocked attackers
- Damage cleanup at CLEANUP step
- Comprehensive test coverage
- Known limitations:
  - ❌ No multiple blockers per attacker
  - ❌ No damage assignment ordering
  - ❌ No first strike / double strike
  - ❌ No trample
  - ❌ No deathtouch
  - ❌ No indestructible
  - ❌ No damage prevention
  - ❌ No combat-damage triggers

---

## 🟡 Pending to Close the Core

---

### 1️⃣ Static Abilities — MVP (Consultative Keywords)
**Goal**
- Support simple always-on rule modifiers

**Included keywords (MVP)**
These keywords are **local, consultative, and non-invasive**:
- Flying
- Reach
- Vigilance

**Why these are included**
- Modify a single validation rule
- Do not affect targeting, costs, or stack behavior
- Do not require replacement effects
- Do not introduce irreversible engine coupling

**Accepted limitations**
- No full 7-layer system
- No dependency resolution
- No ability gain/loss interactions

---

## 🔵 Planned Post-Core Expansions (Explicitly Out of MVP)

These features are **intentionally excluded from the Core MVP**, but are
**explicitly planned** and tracked to avoid blind spots.

---

### Combat Extensions (Post-MVP)

#### Multiple Blockers per Attacker
- Support multiple creatures blocking a single attacker
- Introduce blocker ordering
- Implement attacker-controlled damage assignment order

**Required groundwork**
- Ordered damage assignment
- Partial damage tracking
- Future interaction with trample and deathtouch

---

#### Advanced Combat Keywords
- First Strike / Double Strike
- Trample
- Deathtouch
- Indestructible
- Damage prevention

**Reason**
- Depend on ordered damage assignment
- Interact with state-based actions and replacement effects

---

### Advanced Static Keywords
- Lifelink
- Infect / Poison counters
- Menace

**Reason**
- Interact with damage model, counters, and triggers

---

### Targeting-Altering Keywords
- Hexproof
- Shroud
- Protection
- Ward

**Reason**
- Affect targeting globally
- Require timing guarantees and invalidation rules

---

### Replacement Effects & Advanced Rules
- Damage replacement / redirection
- Damage to planeswalkers
- Regeneration
- Planeswalker uniqueness rule

---

### Full Static Ability Layer System
- Official 7-layer rules
- Dependency resolution
- Timestamp ordering

**Note**
- Acknowledged as required for full Magic fidelity
- Explicitly deferred to avoid premature engine lock-in

---

## 🧩 What Unlocks UI Work

Once the following are completed:
- Mana Pool MVP ✅
- Costs ✅
- Permanent Types MVP ✅
- Power/Toughness + Counters
- Combat MVP

We can safely build:
- Zone UI
- Stack UI
- Priority UI
- Combat UI
- Target selection UI

Without reworking engine fundamentals.

---

## 🛠️ How This Document Is Maintained

- Each PR that closes a block:
  - Marks it as completed
  - References relevant tests
- Known limitations are explicitly documented
- Deferred features remain visible and intentional
- Nothing is removed without discussion

This document is the **single source of truth** for Echomancy’s engine roadmap.
