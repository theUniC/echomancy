# GameSnapshot — UI Layer Contract

This document describes the GameSnapshot system in Echomancy.

## Overview

GameSnapshot is the UI-facing game state representation that provides player-relative, visibility-filtered views of the game. It lives OUTSIDE the core engine and serves as the boundary between the engine and the UI.

## Purpose

GameSnapshot solves ONE problem:

> Present a **player-relative, UI-friendly view** of the game
> without leaking rules or hiding engine truth.

The engine exports everything for everyone with no filtering (via `GameStateExport`).
The UI needs visibility rules, ownership perspective, and readable structure.
GameSnapshot is the **ONLY place** where this transformation is allowed.

## Architecture

```
Game (domain logic)
    ↓
GameStateExport (raw, complete, unfiltered)
    ↓
GameSnapshot (filtered, player-relative, UI-friendly)
    ↓
UI Components (rendering)
```

### Relationship to GameStateExport

- GameSnapshot is **derived entirely** from GameStateExport
- GameSnapshot is **reconstructible** at any time
- GameSnapshot is **discardable** and recreatable
- GameSnapshot contains **NO engine references**

## Design Principles

### What GameSnapshot IS

- ✅ Player-relative (created FOR a specific viewer)
- ✅ Visibility-filtered (hides opponent's hand)
- ✅ UI-friendly (flattened, display-ready structures)
- ✅ Immutable (no mutation after creation)
- ✅ Reconstructible (can be rebuilt from export at any time)

### What GameSnapshot is NOT

- ❌ Not a rules engine
- ❌ Not authoritative
- ❌ Not mutable
- ❌ Not bidirectional
- ❌ Not allowed to guess legality

If the UI needs to know whether something is legal:
👉 it asks the engine (via allowed actions)

## Core Types

### GameSnapshot

The main snapshot type, created for a specific player:

```typescript
type GameSnapshot = {
  viewerPlayerId: string
  publicGameState: PublicGameState
  privatePlayerState: PrivatePlayerState
  opponentStates: readonly OpponentState[]
  visibleStack: StackSnapshot
  uiHints: UIHints | null
}
```

### PublicGameState

Information visible to ALL players:

- Turn number
- Current/active player
- Priority player
- Current phase and step
- Combat summary (if in combat)
- Stack size

### PrivatePlayerState

Full visibility of viewer's zones:

- Life total
- Poison counters
- Mana pool
- **Hand** (full card visibility)
- Battlefield
- Graveyard
- Exile

### OpponentState

Hidden information applied:

- Life total
- Poison counters
- Mana pool (visible in current MVP)
- **Hand size only** (NOT the cards themselves)
- Battlefield (visible)
- Graveyard (visible)
- Exile (visible)

### CardSnapshot

Flattened, display-ready card representation:

- Instance ID
- **Name** (resolved from card registry)
- Types and subtypes
- Controller and owner IDs
- Creature state (tapped, counters, damage, power/toughness)
- Static keywords (FLYING, VIGILANCE, etc.)
- Combat state (attacking, blocking, etc.)

### StackSnapshot

Stack items ordered top to bottom (index 0 = top of stack):

- Source card name
- Controller ID
- Kind (SPELL | ACTIVATED_ABILITY | TRIGGERED_ABILITY)
- Target descriptions (human-readable)

### UIHints

Optional convenience flags derived from engine output:

- Can pass priority
- Highlighted attackers (instance IDs)
- Highlighted blockers (instance IDs)

**IMPORTANT**: UIHints must NOT encode rules logic. They are purely convenience flags derived directly from exported state without interpretation.

**Note**: `canPlayLand` was removed because determining land legality requires rules logic (checking turn, step, lands played). The UI must ask the engine for "allowed actions" instead of inferring legality.

## Usage

### Creating a Snapshot

```typescript
import { createGameSnapshot } from '@/echomancy/infrastructure/ui/GameSnapshot'

// Export raw state from engine
const exportedState = game.exportState()

// Create snapshot for specific player
const snapshot = createGameSnapshot(
  exportedState,
  player1.id,
  cardRegistry
)

// Use snapshot for rendering
renderUI(snapshot)
```

### Card Registry

The card registry resolves card definition IDs to human-readable names:

```typescript
type CardRegistry = {
  getCardName(cardDefinitionId: string): string
}
```

This allows the UI to display "Lightning Bolt" instead of "card-def-12345".

## Visibility Rules

### Viewer Perspective

- Viewer sees **all cards** in their own hand
- Viewer sees **all zones** they own
- Viewer has **full visibility** of their private state

### Opponent Perspective

- Viewer sees opponent's **hand size only**
- Viewer does NOT see opponent's hand cards
- Viewer sees opponent's **battlefield** (public zone)
- Viewer sees opponent's **graveyard** (public zone)
- Viewer sees opponent's **exile** (public zone)

### Public Information

- Turn structure
- Priority
- Stack contents (all items visible)
- Combat state (attackers/blockers)
- Life totals
- Mana pools (in current MVP)

## Invariants

1. **Immutability**: Snapshot cannot be mutated after creation
2. **No Engine References**: Snapshot contains only plain data
3. **Reconstructibility**: Same export always produces same snapshot for same viewer
4. **Visibility Correctness**: Hidden information is never leaked
5. **No Rules Logic**: Snapshot transformation contains no game rules

## Known Limitations (MVP)

- No fog-of-war animation logic
- No timing animations
- No speculative previews
- No prediction of outcomes
- Poison counters always 0 (not yet implemented)
- Exile zone always empty (not yet implemented)
- Library not included (not yet implemented)

## Testing

The GameSnapshot system is tested for:

- Correct visibility filtering
- Player perspective symmetry
- Immutability
- Reconstructibility
- Complete coverage of all zones
- Proper combat state representation
- Stack snapshot accuracy
- UI hints derivation

See `src/echomancy/infrastructure/ui/__tests__/GameSnapshot.test.ts` for comprehensive test suite.

## Future Work

### Planned Enhancements

1. **Snapshot Diffing**: For animations between snapshots
2. **Snapshot Versioning**: For replay compatibility
3. **Performance Optimizations**: Caching, memoization
4. **Accessibility Metadata**: For screen readers
5. **Spectator Mode**: Snapshots with full visibility
6. **Replay Mode**: Historical snapshots

### NOT Planned

- Rules inference
- Speculative execution
- Predictive UI
- Auto-play logic

## Integration with UI

The UI must:

1. Use **ONLY** GameSnapshot for rendering
2. **NEVER** access GameStateExport directly
3. **NEVER** access Game domain objects
4. **NEVER** infer rules from snapshot data
5. **ALWAYS** ask engine for legal actions

GameSnapshot is a **mirror**, not a simulator.

## Maintenance Rules

- If it feels like a rule, it belongs in the engine
- If it feels like UI convenience, it goes here
- When in doubt, keep GameSnapshot dumb
- Snapshot logic must be stateless and pure

## Summary

GameSnapshot is the **only safe boundary** between engine and UI. It provides:

- Player-relative views
- Visibility filtering
- UI-friendly data structures
- No rules coupling
- Complete testability

This separation ensures:

- Engine remains UI-agnostic
- Multiple UIs can consume same export
- Replays have full information
- AI/bots can see complete state
- UI changes don't affect rules

End of document.
