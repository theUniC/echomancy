# Game Snapshot

UI-facing game state representation that provides player-relative, visibility-filtered views of the game.

## Key Concepts

- **Player-relative**: Created for a specific viewer, shows their perspective
- **Visibility-filtered**: Hides opponent's hand cards, shows only hand size
- **UI-friendly**: Flattened, display-ready data structures
- **Derived**: Built entirely from GameStateExport (see `game-state-export.md`)
- **Immutable**: Cannot be mutated after creation
- **Reconstructible**: Same export always produces same snapshot for same viewer

## How It Works

GameSnapshot sits between the raw engine export and the UI layer:

```
Game → GameStateExport → GameSnapshot → UI Components
```

The snapshot transforms the complete, unfiltered export into a view appropriate for a specific player. It contains:

- **PublicGameState**: Turn number, active player, priority, phase/step, combat state
- **PrivatePlayerState**: Viewer's full state (hand cards visible, life, mana, all zones)
- **OpponentStates**: Hidden information applied (hand size only, no cards)
- **VisibleStack**: Stack items with human-readable names and targets
- **UIHints**: Optional convenience flags (can pass priority, highlighted creatures)

**Implementation**: `src/echomancy/infrastructure/ui/GameSnapshot.ts`
**Tests**: `src/echomancy/infrastructure/ui/__tests__/GameSnapshot.test.ts`

## Rules

### Creation
- Snapshot is created by calling `createGameSnapshot(exportedState, playerId, cardRegistry)`
- Requires a GameStateExport, viewer player ID, and card registry for name resolution
- CardRegistry resolves card definition IDs to display names ("Lightning Bolt" vs "card-def-12345")

### Visibility
- Viewer sees ALL cards in their own hand
- Viewer sees opponent's hand SIZE only (not cards)
- All public zones visible to everyone (battlefield, graveyard, exile)
- Stack contents fully visible to all players

### Constraints
- NO engine references allowed (plain data only)
- NO rules logic in transformation (no legality checks)
- NO mutation after creation
- UI MUST use snapshot for rendering (never access GameStateExport or Game directly)

### UIHints Limitations
- UIHints must contain NO rules logic
- Must be derived directly from exported state without interpretation
- Cannot encode legality checks (e.g., "can play land" removed because it requires rules logic)
- For legality, UI must ask engine for "allowed actions"
