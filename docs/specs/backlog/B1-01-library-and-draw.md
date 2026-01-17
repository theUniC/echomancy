# Library Zone and Draw Card Action

## Overview

Implement the **library zone** (deck) and **draw card action** to enable players to draw cards from their library into their hand. This is a fundamental game mechanic required before implementing mulligan, opening hands, and normal draw steps.

### Design Goals
- Library zone exists as an ordered collection of cards per player
- Players can draw cards from top of library to hand
- Handle empty library (draw from empty library loses the game)
- Support for future mulligan and opening hand systems

### Relationship to Other Systems
- **Zones System**: Adds library as a new player zone alongside hand, battlefield, graveyard
- **Game Actions**: Adds DRAW_CARD action to the action system
- **Game State Export**: Extends PlayerStateExport to include library zone
- **State-Based Actions**: Adds check for empty library draw (player loses)

## User Stories

### As a player
- I want to draw cards from my library so I can access new cards during the game
- I want to see how many cards remain in my library so I can plan ahead
- I want the game to end if I draw from an empty library (following MTG rules)

### As a game developer
- I need library to be an ordered zone so card order matters (for effects like "look at top N cards")
- I need DRAW_CARD to be a proper game action so it goes through validation
- I need to support pre-constructed decks for testing before implementing deck builder

## Player Experience

### What the player sees
- Each player has a library zone (represented as a card count in UI MVP)
- Library count decreases when cards are drawn
- Drawn cards appear in the player's hand

### What actions can they take
- Player cannot directly interact with library in MVP
- Drawing happens automatically during draw step
- Drawing can be triggered by card effects (future)

### What feedback do they receive
- Library count updates when cards are drawn
- If drawing from empty library, player loses immediately
- Clear visual indication of library size

### Player flow
1. Game starts with library containing preset cards (ordered)
2. During draw step, current player automatically draws a card
3. Card moves from top of library to hand
4. Library count decreases by 1
5. If library becomes empty and player tries to draw, they lose

## Game Rules & Mechanics

### Library Zone Rules
- Each player has their own library zone
- Library is ordered (top card is index 0, bottom card is last)
- Library is hidden information (opponent cannot see contents)
- Library size is public information (both players can see count)

### Draw Card Rules
- Drawing takes a card from the top of library (index 0)
- Card is removed from library and added to hand
- Drawing is atomic (cannot be interrupted mid-draw)
- Drawing from empty library causes the player to lose immediately

### Edge Cases
- **Draw from empty library**: Player loses immediately (state-based action)
- **Multiple draws**: Each draw is processed individually, in sequence
- **Simultaneous draws**: Not possible in MTG rules (one active player at a time)
- **Library becomes empty but no draw attempted**: Game continues normally

### Interaction with Other Mechanics
- **Turn Structure**: Draw step automatically triggers DRAW_CARD for active player
- **State-Based Actions**: After any draw, check if player drew from empty library
- **Zone Changes**: Drawing emits ZONE_CHANGED event (from LIBRARY to HAND)
- **Future**: Shuffle, mill, look at top N cards will use library zone

## MVP Scope

### What This Feature Includes
1. Add library zone to PlayerState
2. Add DRAW_CARD action to GameActions
3. Implement draw card logic in Game.apply()
4. Add library to GameStateExport
5. Handle "draw from empty library loses" as state-based action
6. Update turn structure to draw at draw step
7. Tests for:
   - Normal draw
   - Draw from empty library
   - Draw step integration
   - Library count tracking

### What This Feature Does NOT Include
- Deck builder (out of scope for now)
- Deck shuffling (defer to when needed)
- Mulligan system (separate feature)
- Opening hand draw (separate feature)
- Effects that manipulate library (look, search, mill - future)
- Library UI display (separate UI phase)
- Library order randomization at game start (can be added later)

### MVP Implementation Details

#### Game Setup
For MVP, games are created with a pre-constructed library per player:

```typescript
// Example: Game initialization includes decks
const game = new Game({
  gameId: "game-1",
  players: [
    { id: "p1", deck: ["card-1", "card-2", ...] },
    { id: "p2", deck: ["card-3", "card-4", ...] }
  ]
})
```

- Deck is provided as array of CardDefinition IDs
- Order is preserved (first in array = top of library)
- No shuffling in MVP (deterministic for testing)
- Shuffle can be added later without breaking changes

#### Test Decks
For testing, predefined test decks will be created:

```typescript
// Test helper example
const TEST_DECK = [
  "test-creature-1",
  "test-land-1",
  "test-instant-1",
  // ... etc
]
```

## Acceptance Criteria

### Functional Requirements
- [ ] PlayerState includes library zone with ordered cards
- [ ] DRAW_CARD action exists and is handled by Game.apply()
- [ ] Drawing moves top card from library to hand
- [ ] Library count decreases after draw
- [ ] Drawing from empty library causes player to lose
- [ ] Draw step automatically draws card for active player
- [ ] ZONE_CHANGED event fires when card drawn (LIBRARY → HAND)

### Technical Requirements
- [ ] GameStateExport includes library zone
- [ ] Library is ordered (not just a set)
- [ ] Draw validates player has priority (if action, not automatic)
- [ ] Tests cover normal draw and empty library draw
- [ ] Tests verify library order preservation
- [ ] Tests verify state-based action for empty library draw

### Integration Requirements
- [ ] Turn structure calls draw automatically at DRAW step
- [ ] Game initialization accepts deck parameter per player
- [ ] Test helpers exist for creating games with custom decks
- [ ] Documentation updated (zones-and-cards.md)

## Out of Scope

The following are explicitly deferred to future features:

### Deferred to Later
1. **Deck Builder** - Players will not build custom decks in MVP
2. **Shuffle Library** - Deterministic order for testing; shuffle added when needed
3. **Mulligan System** - Separate feature (requires opening hand first)
4. **Opening Hand Draw** - Separate feature (draw 7 at game start)
5. **Library Manipulation Effects**:
   - "Look at top N cards"
   - "Search library"
   - "Mill" (put top N cards into graveyard)
   - "Put card on top/bottom of library"
6. **Library UI** - UI phase will handle visual representation
7. **Draw triggers** - "When you draw a card" triggered abilities (future)
8. **Replacement effects** - "Instead of drawing, do X" (future)

### Why Deferred
- Mulligan and opening hand are complex enough to be separate features
- Library manipulation effects require more complex targeting/selection UI
- Shuffle algorithm can be added without breaking existing draw system
- UI phase will handle all visual aspects of library

### Dependencies for Future Features
- **Opening Hand**: Requires this feature + shuffle + DRAW_CARD action
- **Mulligan**: Requires opening hand + shuffle + library manipulation
- **Deck Builder**: Requires card database + deck validation system
- **Draw Triggers**: Requires triggered ability system (already exists)

## Technical Notes

### Action Structure
```typescript
type DrawCard = {
  type: "DRAW_CARD"
  playerId: string
}
```

### State Structure Changes
```typescript
// PlayerState.ts
export type PlayerState = {
  hand: Zone
  battlefield: Zone
  graveyard: Zone
  library: Zone // NEW
}
```

### Game Loss Condition
When a player attempts to draw from an empty library, they lose immediately:
- Check library.cards.length === 0 before draw
- If empty, mark player as having lost
- Game ends if only one player remains

### Event Emission
Drawing emits ZONE_CHANGED event:
```typescript
{
  type: "ZONE_CHANGED",
  cardInstanceId: drawnCard.instanceId,
  fromZone: "LIBRARY",
  toZone: "HAND",
  controllerId: playerId
}
```

This enables future "when you draw" triggered abilities without refactoring.

## Success Metrics

This feature is successful when:
1. Players can draw cards from library during draw step
2. Tests verify correct library ordering and draw behavior
3. Game correctly handles empty library as loss condition
4. Future features (mulligan, opening hand) can build on this foundation
5. No breaking changes required when adding shuffle or deck builder

## Implementation Guidance

### Recommended Implementation Order
1. Add library to PlayerState and Zone types
2. Add library to GameStateExport
3. Add DRAW_CARD action to GameActions
4. Implement draw logic in Game.apply()
5. Add empty library check (state-based action)
6. Integrate draw into turn structure (DRAW step)
7. Update Game constructor to accept deck parameter
8. Write tests for all scenarios
9. Update documentation

### Key Considerations
- Library is **ordered** - use array, not set
- Draw is **atomic** - card moves completely or not at all
- Empty library draw is **immediate loss** - no recovery
- Future-proof for shuffle, mulligan, opening hand
