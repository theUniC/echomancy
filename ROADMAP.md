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

## Game State Export
- Engine exposes a **pure, serializable state**
- No UI concepts inside the core

## GameSnapshot (UI Layer)
- Built outside the engine
- Player-relative visibility
- Hidden information filtering
- Read-only representation

---

# 🎮 PART II — UI MVP (REQUIRED)

The UI is part of the MVP.  
Without it, Echomancy is **not playable**.

---

## 🟢 UI MVP Scope

### 1️⃣ Zone UI
- Hand (current player)
- Battlefield (both players)
- Graveyard (both players)

---

### 2️⃣ Stack UI
- Visible stack
- Order of spells and abilities
- Source and controller

---

### 3️⃣ Priority UI
- Active player indicator
- Pass priority action
- Visual priority ownership

---

### 4️⃣ Turn & Phase UI
- Current turn owner
- Current phase / step
- Visual step transitions

---

### 5️⃣ Combat UI (MVP)
- Attacker selection
- Blocker selection (1-to-1)
- Damage resolution visualization
- Life total updates

---

### 6️⃣ Target Selection UI
- Valid target highlighting
- Target confirmation
- Cancel / invalid target feedback

---

### 7️⃣ Action UI
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
- Spectator mode

---

# 🔵 PART III — Post-MVP Expansions (Tracked)

These features are **planned, visible, and intentional**.

---

## Engine Extensions
- Multiple blockers
- Ordered damage assignment
- Advanced combat keywords
- Full counter system
- Planeswalker loyalty
- Replacement effects
- 7-layer system

---

## UI Extensions
- Animations
- Advanced tooltips
- Stack inspection
- Combat replay
- Deck builder
- Rules inspector / debug mode

---

## 🛠️ Maintenance Rules

- No hidden scope
- Deferred ≠ forgotten
- All limitations documented
- One roadmap, one truth

This document defines **what Echomancy is** and **what it will become**.
