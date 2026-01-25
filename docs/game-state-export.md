# Game State Export

Raw, complete, unfiltered representation of game state for serialization, replay, and transformation to player-specific views.

## Key Concepts

- **Complete**: Includes ALL game state, including hidden information (opponent hands, libraries)
- **Neutral**: No UI concepts, no player perspective, no filtering
- **Deterministic**: Same state always produces same export
- **Plain data**: Serializable structures only (no methods, no engine references)
- **Intermediate**: Not for direct UI consumption (use GameSnapshot for that)

## How It Works

GameStateExport is the boundary between the game engine and all external consumers:

```
Game (domain logic) → GameStateExport → [GameSnapshot | Network | Replay | AI]
```

The export contains everything needed to reconstruct or observe the complete game state:

- **Game metadata**: ID, turn number, active player, current step
- **Priority**: Who has priority (null if none)
- **Turn order**: Player IDs in turn order
- **Player states**: Life totals, mana pools, lands played, all zones (including hidden hand and library)
- **Stack**: All stack items with sources, controllers, targets
- **Scheduled steps**: Extra steps scheduled (e.g., extra combat phases)

**For player-specific views with visibility filtering**: See `game-snapshot.md`

**Implementation**: `Game.exportState()` in `src/echomancy/domainmodel/game/Game.ts`
**Tests**: `src/echomancy/domainmodel/game/__tests__/GameStateExport.test.ts`

## Rules

### Export Properties
- Immutable after creation
- Contains every card exactly once
- All references are IDs (instance IDs, player IDs, card definition IDs)
- No circular references
- Fully type-safe with TypeScript definitions

### What to Export
- ALL zones for ALL players (hand, library, battlefield, graveyard, exile)
- Complete creature state (tapped, attacking, counters, damage)
- Complete stack state (spells, abilities, targets)
- Scheduled steps and resume points
- Mana pools and land plays

### What NOT to Export
- Allowed actions or UI hints (use GameSnapshot)
- Legality checks or validation state
- Derived information that can be recomputed
- Engine references or domain objects

### Usage
```typescript
const game = Game.start({ id, players, startingPlayerId })
// ... play some turns ...
const exported = game.exportState()
```
