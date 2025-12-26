# Game State Export

This document describes the game state export system in Echomancy.

## Overview

The game state export system provides a clean boundary between the game engine and external consumers (UI, network, replay systems, AI).

## Design Philosophy

The export is:
- **Complete**: Includes all game state, even hidden information (hands, libraries)
- **Neutral**: No UI concepts or helpers
- **Unfiltered**: No visibility rules applied
- **Deterministic**: Same state always produces same export
- **Plain Data**: Serializable structures only (no methods)

## Architecture

```
Game (domain logic)
    ↓
GameStateExport (neutral intermediate representation)
    ↓
GameSnapshot (UI layer with visibility rules - not yet implemented)
```

The export is an intermediate representation. It is NOT:
- A UI model
- A snapshot with visibility filtering
- A view model
- A network protocol
- Player-specific

## Export Structure

### GameStateExport

Top-level export containing:
- `gameId`: Unique game identifier
- `currentTurnNumber`: Turn counter (increments on round completion)
- `currentPlayerId`: Active player
- `currentStep`: Current game step
- `priorityPlayerId`: Player with priority (null if none)
- `turnOrder`: Array of player IDs in turn order
- `players`: Record of player states (keyed by player ID)
- `stack`: Array of stack items
- `scheduledSteps`: Array of scheduled extra steps
- `resumeStepAfterScheduled`: Optional resume point

### PlayerStateExport

Per-player data:
- `lifeTotal`: Player's life total
- `manaPool`: Mana available in each color (W, U, B, R, G, C)
- `playedLandsThisTurn`: Land plays this turn (0 for non-active players)
- `zones`: Player's zones (hand, battlefield, graveyard, library?)

### CardInstanceExport

Card representation:
- `instanceId`: Unique instance identifier
- `ownerId`: Player who owns this card
- `controllerId`: Player who controls this card
- `cardDefinitionId`: Reference to card definition
- `name`: Card name (for debugging)
- `types`: Array of card types
- `staticAbilities`: Static ability keywords (if any)
- `power`: Base power (for creatures)
- `toughness`: Base toughness (for creatures)
- `creatureState`: Creature-specific state (if applicable)
- `planeswalkerState`: Planeswalker state (MVP placeholder)

### CreatureStateExport

Creature-specific state:
- `isTapped`: Tap status
- `isAttacking`: Attacking status
- `hasAttackedThisTurn`: Attack history
- `power`: Current power (including counters)
- `toughness`: Current toughness (including counters)
- `counters`: Counter types and amounts
- `damageMarkedThisTurn`: Accumulated damage
- `blockingCreatureId`: Creature being blocked (null if none)
- `blockedBy`: Blocking creature (null if unblocked)

### StackItemExport

Stack item representation:
- `kind`: "SPELL" | "ACTIVATED_ABILITY" | "TRIGGERED_ABILITY"
- `sourceCardInstanceId`: Card instance ID that created this stack item
- `sourceCardDefinitionId`: Card definition ID (for UI name resolution)
- `controllerId`: Player who controls this
- `targets`: Array of target instance IDs

## Usage

### Exporting State

```typescript
const game = Game.start({ id, players, startingPlayerId })

// Play some turns...

const exported = game.exportState()
```

### Export Properties

The export is:
- **Immutable**: No mutation after creation
- **Complete**: Every card referenced exists exactly once
- **Type-safe**: Full TypeScript type definitions
- **Testable**: Easy to assert on specific properties

## What This Is NOT

Do NOT use the export to:
- Hide information from players (use GameSnapshot for that)
- Add "allowed actions" or UI helpers
- Filter by visibility rules
- Add validation logic

The export is raw, complete game state. All filtering and UI logic belongs in higher layers.

## Future Work

### GameSnapshot (Planned)

A future layer will consume `GameStateExport` and:
- Apply visibility rules (hide opponent's hand)
- Add UI helpers
- Provide player-specific views
- Support network serialization

This separation ensures:
- Game engine remains UI-agnostic
- Multiple UIs can consume the same export
- Replays have full information
- AI/bots can see complete state

## Testing

The export is tested for:
- Completeness (all information present)
- Determinism (same state produces same export)
- Non-mutation (export doesn't change game state)
- Accuracy (reflects actual game state)
- Invariants (no duplicate cards, consistent references)

See `src/echomancy/domainmodel/game/__tests__/GameStateExport.test.ts` for comprehensive test suite.
