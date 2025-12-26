# Echomancy — Engine & UI Roadmap (Living Document)

This document defines the roadmap for **Echomancy as a whole**:
- the Core Rules Engine
- the Engine ↔ UI boundary
- the minimal playable UI
- and planned post-MVP expansions

This is a **single source of truth** document.

Echomancy aims to be:

* 👉 an open, transparent, and fair Magic rules engine  
* 👉 focused on correctness, testability, and long-term maintainability  
* 👉 not constrained by commercial shortcuts or opaque rule systems  

---

## 🎯 Definition of “Project MVP Complete”

Echomancy is considered **MVP-complete** when:

- The core engine models real Magic rules correctly
- A full turn cycle works end to end
- A minimal UI allows two players to play a real game
- No major refactor is required to extend either engine or UI

---

# 🧩 PART I — Core Rules Engine

## 🟢 Core Engine Status: **CLOSED**

The Core Rules Engine is complete and validated.

### What “CLOSED” Means

“CLOSED” means:
- The engine is sufficient to play real Magic games via a UI
- All implemented rules are considered stable
- Future work extends the engine but does not invalidate the UI contract

“CLOSED” does **NOT** mean:
- All Magic rules are implemented
- Full rules fidelity has been reached
- No further engine work will ever be required

---

### Turns and Phases
- Full turn structure
- Extra phases supported
- Correct resume of normal flow
- Test coverage for real scenarios

---

### Priority and Stack
- Priority alternation
- Stack resolution
- Spells and abilities on the stack
- Deterministic resolution
- CI green

---

### Cards and Zones
- Hand / Battlefield / Graveyard
- Play land (no stack)
- Cast spell (uses stack)
- Correct zone transitions

---

### ETB (Enter the Battlefield)
- ETB triggers when permanents resolve
- Known limitation: simplified targeting

---

### Creatures (MVP)
- Tapped / attacking / attacked-this-turn state
- Declare attackers
- Attack restrictions
- Correct reset on phase and turn change

---

### Abilities
- Activated abilities
- Triggered abilities
- Stack integration
- Real card tests (Elves)

---

### Mana Pool (MVP)
- Per-player mana pool (W, U, B, R, G, C)
- Add / spend mana
- Pool clears at CLEANUP
- Error handling and tests

---

### Costs (Beyond Mana)
- Explicit cost model
- Cost / effect separation
- Atomic cost payment
- Supported:
  - ManaCost
  - TapSelfCost
  - SacrificeSelfCost
- Reusable by spells and abilities

---

### Permanent Types (MVP)
- Creature
- Land
- Artifact
- Enchantment
- Planeswalker (placeholder)

Known limitations:
- No loyalty yet
- No attachments (auras / equipment)

---

### Power / Toughness + Counters (MVP)
- Base power / toughness
- +1/+1 counters
- Current P/T calculation

Explicitly deferred:
- -1/-1 counters
- Poison counters
- Charge counters
- Loyalty counters

---

### Combat Resolution (MVP)
- Declare attackers
- Declare blockers (1-to-1 only)
- Simultaneous damage
- Creature destruction
- Damage to players
- Cleanup

Known limitations:
- No multiple blockers
- No first strike / trample / deathtouch
- No combat damage triggers

---

### Static Abilities (MVP — Consultative)
- Flying
- Reach
- Vigilance

Rules-only, consultative keywords:
- No stack interaction
- No layers
- No gain/loss

---

# 🔒 Core Boundary: Engine ↔ UI

This boundary is **intentional and enforced**.

## Engine Responsibilities
- Own all game rules
- Own validation
- Own state transitions
- Be UI-agnostic

## UI Responsibilities
- Never infer rules
- Never mutate game state directly
- Only reflect engine output

---

## Game State Export ✅ IMPLEMENTED
- Engine exposes a **pure, serializable state** via `Game.exportState()`
- Complete export including all hidden information
- Neutral, deterministic data structure

## GameSnapshot (UI Layer) — TODO
- Built outside the engine
- Player-relative visibility
- Hidden information filtering
- Read-only representation

---

# 🎮 PART II — UI MVP (REQUIRED)

The UI is part of the MVP.

---

## 🟢 UI MVP Scope

### Zone UI
- Hand
- Battlefield
- Graveyard

### Stack UI
- Visible stack
- Order, source, controller

### Priority UI
- Active player indicator
- Pass priority

### Turn & Phase UI
- Turn owner
- Phase / step display

### Combat UI
- Attacker selection
- Blocker selection (1-to-1)
- Damage visualization

### Target Selection UI
- Valid target highlighting
- Confirmation / cancellation

### Action UI
- Play land
- Cast spell
- Activate ability
- Declare attacker / blocker
- End turn

---

## ❌ Explicitly NOT in UI MVP
- Deck builder
- Matchmaking
- Replays
- Animations
- Sound

---

# ⏭️ PART III — NEXT: Remaining Magic Features (Exhaustive Reference)

This section lists **all known Magic features not yet implemented**.

Legend:
- **[UI-REQ]** Required for specific UI features
- **[CORE-EXT]** Engine extension (does not break UI)
- **[COMPAT]** Required for full Magic rules compatibility

---

## Zones
- Exile zone [UI-REQ]
- Command zone [COMPAT]

## Counters
- -1/-1 counters [COMPAT]
- Poison counters [COMPAT]
- Charge counters [COMPAT]
- Loyalty counters [CORE-EXT]

## Combat Extensions
- Multiple blockers per attacker [CORE-EXT]
- Ordered damage assignment [CORE-EXT]

## Keywords
- Haste [CORE-EXT]
- First Strike / Double Strike [CORE-EXT]
- Trample [CORE-EXT]
- Deathtouch [CORE-EXT]
- Indestructible [CORE-EXT]

## Targeting & Protection
- Hexproof [COMPAT]
- Shroud [COMPAT]
- Protection [COMPAT]
- Ward [COMPAT]

## Replacement & State Rules
- Replacement effects [COMPAT]
- Damage prevention [COMPAT]
- Damage redirection [COMPAT]
- State-based actions (full) [COMPAT]

## Continuous Effects
- Lords and global modifiers [CORE-EXT]
- Ability gain/loss [CORE-EXT]
- Full 7-layer system [COMPAT]

---

## Design Philosophy

Echomancy prioritizes:
1. Rules correctness over shortcuts
2. Explicit modeling over inference
3. Engine determinism over UI convenience
4. Transparency over opaque expert systems

Echomancy explicitly avoids:
- Arena-style hidden logic
- UI-driven rule decisions
- Heuristic card-text parsing
- Premature expert-system approaches

---

## 🛠️ Maintenance Rules

- No hidden scope
- Deferred ≠ forgotten
- All limitations documented
- One roadmap, one truth

This document defines **what Echomancy is** and **what it will become**.
