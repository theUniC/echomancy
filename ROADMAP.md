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
- Known limitations (no loyalty yet, no attachments)

### Power / Toughness + Counters (MVP)
- Base power and toughness for creatures
- +1/+1 counter support
- Counter addition and removal with validation
- Current power/toughness calculation
- Comprehensive test coverage (28 tests)
- Known limitations:
  - No damage tracking (TODO: implement damage model)
  - No static abilities (TODO: implement continuous effects)
  - No layer system (TODO: implement 7-layer system)
  - No temporary modifiers (TODO: implement duration tracking)
  - Only +1/+1 counters supported (TODO: -1/-1, poison, etc.)

---

## 🟡 Pending to Close the Core

---

### 1️⃣ Combat — Resolution MVP
**Goal**
- Make combat fully resolvable

**Scope**
- Declare attackers
- Declare blockers
- Damage assignment
- Damage resolution
- Creature destruction
- Damage to players

**Notes**
- Uses Power/Toughness
- No advanced combat abilities yet

---

### 2️⃣ Static Abilities — MVP (Consultative Keywords)
**Goal**
- Support simple always-on rules modifiers

**Included keywords (MVP)**
These keywords are **local, consultative, and non-invasive**:
- Flying
- Reach
- Vigilance
- (Optionally) simple First Strike / Trample

**Why these are included**
- They modify a single rule or validation
- They do not affect targeting, costs, or stack behavior
- They do not require replacement effects
- They do not force irreversible engine decisions

**Accepted limitations**
- No full 7-layer system
- No dependency resolution
- No dynamic ability loss/gain interactions

---

## 🔵 Planned Post-Core Expansions (Explicitly Out of MVP)

These features are **intentionally excluded from the Core MVP**, but are
**explicitly planned** and will be addressed in later milestones.

### Advanced Static Keywords
These keywords affect multiple subsystems and require mature infrastructure:

- Deathtouch (redefines lethal damage)
- Lifelink (post-damage effects)
- Infect / Poison counters
- Double Strike
- Indestructible
- Menace

**Reason**
- Require a finalized damage model
- Interact with counters, triggers, and replacement effects

---

### Targeting-Altering Keywords
- Hexproof
- Shroud
- Protection
- Ward

**Reason**
- Affect targeting rules globally
- Require invalidation logic and timing guarantees
- Depend on future replacement-effect systems

---

### Replacement Effects & Advanced Rules
- Damage replacement/prevention
- Redirection to planeswalkers
- Regeneration
- Full planeswalker uniqueness rule

---

### Full Static Ability Layer System
- Official 7-layer rules
- Dependency resolution
- Timestamp ordering

**Note**
- The 7-layer system is acknowledged as necessary for full Magic support
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
