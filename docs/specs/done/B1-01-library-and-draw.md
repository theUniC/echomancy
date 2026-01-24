# Library Zone and Draw Card Action

## Overview

Implement the **library zone** (deck) and **draw card action** to enable players to draw cards from their library into their hand. This is a fundamental game mechanic required before implementing mulligan, opening hands, and normal draw steps.

### Design Goals
- Library zone exists as an ordered collection of cards per player
- Players can draw cards from top of library to hand
- Handle empty library draw correctly per MTG rules
- Support for future mulligan and opening hand systems

### Relationship to Other Systems
- **Zones System**: Adds library as a new player zone alongside hand, battlefield, graveyard
- **Game Actions**: Adds DRAW_CARD action for card effects that cause draws
- **Turn Structure**: Automatic draw during draw step (turn-based action)
- **Game State Export**: Extends PlayerStateExport to include library zone
- **State-Based Actions**: Adds check for empty library draw (player loses)

## User Stories

### As a player
- I want to draw cards from my library so I can access new cards during the game
- I want to see how many cards remain in my library so I can plan ahead
- I want the game to handle empty library correctly (following MTG rules)

### As a game developer
- I need library to be an ordered zone so card order matters (for effects like "look at top N cards")
- I need to distinguish between automatic draws (draw step) and effect-triggered draws
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
- If drawing from empty library, player loses
- Clear visual indication of library size

### Player flow
1. Game starts with library containing preset cards (ordered)
2. During draw step, current player automatically draws a card
3. Card moves from top of library to hand
4. Library count decreases by 1
5. If player attempts to draw from empty library, they lose

## Game Rules & Mechanics

### Library Zone Rules (per MTG Comprehensive Rules)
- Each player has their own library zone
- Library is ordered (top card drawn first)
- Library is hidden information (opponent cannot see contents)
- Library size is public information (both players can see count)

### Draw Card Rules
- Drawing takes a card from the top of library
- Card is removed from library and added to hand
- Drawing is atomic (cannot be interrupted mid-draw)

### Empty Library Draw (MTG Rules 121.4, 704.5b)
**Critical**: A player who attempts to draw from an empty library loses the game.

The correct sequence per MTG rules:
1. Player attempts to draw (the attempt always happens)
2. If library is empty, the draw fails but the attempt is recorded
3. State-based actions check if a player attempted to draw from empty library
4. If so, that player loses the game
5. The "attempted draw from empty library" flag is cleared after SBAs resolve (so only draws since the last SBA check trigger the loss)

**Why this matters**: The draw attempt must happen (even if it fails) to support future replacement effects like "If you would draw a card, instead..." and triggers like "Whenever you draw a card..."

### Two Types of Drawing

**Turn-based draw (automatic)**:
- Happens during the DRAW step
- Per MTG 504.1: "This turn-based action doesn't use the stack"
- Not a player action - happens automatically
- Cannot be responded to

**Effect-triggered draw (DRAW_CARD action)**:
- Triggered by card effects ("draw 3 cards")
- Goes through the action system
- Can be part of spell/ability resolution

### Edge Cases
- **Draw from empty library**: Attempt happens, then SBA causes loss
- **Multiple draws**: Each draw is processed individually, in sequence
- **Library becomes empty but no draw attempted**: Game continues normally

### Interaction with Other Mechanics
- **Turn Structure**: Draw step triggers automatic draw for active player
- **State-Based Actions**: Checks for "attempted draw from empty library" flag
- **Zone Changes**: Drawing emits ZONE_CHANGED event (from LIBRARY to HAND)
- **Future**: Shuffle, mill, look at top N cards will use library zone

## MVP Scope

### What This Feature Includes
1. Library zone as part of PlayerState (follows existing zone entity pattern)
2. DRAW_CARD action for effect-triggered draws
3. Automatic draw during draw step (turn-based, not an action)
4. Library included in GameStateExport (note: schema already has optional library field)
5. "Draw from empty library loses" via state-based actions (flag-based timing)
6. Ability to set player decks before game starts

### What This Feature Does NOT Include
- Deck builder (out of scope for now)
- Deck shuffling (defer to when needed)
- Mulligan system (separate feature)
- Opening hand draw (separate feature - B1-02)
- Effects that manipulate library (look, search, mill - future)
- Library UI display (separate UI phase)

## Acceptance Criteria

### Functional Requirements
- [ ] Each player has a library zone that holds cards in order
- [ ] Players can have their deck set before the game starts
- [ ] During draw step, active player automatically draws one card
- [ ] Card effects can cause players to draw cards (DRAW_CARD action)
- [ ] Drawing moves top card from library to hand
- [ ] Library count decreases after each draw
- [ ] Attempting to draw from empty library causes player to lose (via SBA)
- [ ] The draw attempt is recorded before SBA checks (correct timing per MTG rules)
- [ ] ZONE_CHANGED event fires when card drawn (LIBRARY → HAND)

### Query Requirements
- [ ] Can query library card count for any player
- [ ] Can export library in game state (card count, hidden contents)

### Test Requirements
- [ ] Normal draw moves card from library to hand
- [ ] Draw from empty library triggers loss via SBA
- [ ] Multiple sequential draws work correctly
- [ ] Library order is preserved (top card drawn first)
- [ ] Draw step automatic draw works
- [ ] DRAW_CARD action works for card effects

## Out of Scope

### Deferred to Later
1. **Deck Builder** - Players will not build custom decks in MVP
2. **Shuffle Library** - Deterministic order for testing; shuffle added when needed
3. **Mulligan System** - Separate feature (requires opening hand first)
4. **Opening Hand Draw** - Separate feature (draw 7 at game start) - B1-02
5. **Library Manipulation Effects**: Look, search, mill, reorder
6. **Library UI** - UI phase will handle visual representation
7. **Draw triggers** - "When you draw a card" triggered abilities
8. **Replacement effects** - "Instead of drawing, do X"

### Dependencies for Future Features
- **Opening Hand (B1-02)**: Requires this feature + shuffle
- **Mulligan**: Requires opening hand + shuffle + library manipulation
- **Deck Builder**: Requires card database + deck validation system
- **Draw Triggers**: Requires triggered ability system (already exists)

## Success Metrics

This feature is successful when:
1. Players can draw cards from library during draw step
2. Game correctly handles empty library per MTG rules (attempt → SBA → loss)
3. Distinction between turn-based draw and effect-triggered draw is clear
4. Future features (mulligan, opening hand) can build on this foundation
5. No breaking changes required when adding shuffle or deck builder

---

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-24
**Completed**: 2026-01-24
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Library Entity ✅
- [x] Create `src/echomancy/domainmodel/game/entities/Library.ts` with immutable pattern
- [x] Implement `empty()`, `fromCards()` factory methods
- [x] Implement `drawFromTop()` returning card and new Library instance
- [x] Implement `peekTop(n)`, `addToBottom()`, `addToTop()`, `count()`, `isEmpty()`
- [x] Create unit tests for Library entity
- [x] Run tests, lint, format

#### Phase 2: PlayerState and Game Integration ✅
- [x] Update `PlayerState.ts` to include `library: Library`
- [x] Update `Game.addPlayer()` to initialize empty library
- [x] Add `setDeck(playerId, cards)` method for pre-game deck setup
- [x] Add tracking for "attempted draw from empty library" flag per player
- [x] Add `getLibraryCount(playerId)` query method
- [x] Run tests, lint, format

#### Phase 3: Draw Card Implementation ✅
- [x] Implement actual `drawCards(playerId, amount)` replacing no-op
- [x] Move top card from library to hand with ZONE_CHANGED event
- [x] Handle empty library draw (set flag, don't immediately lose)
- [x] Handle multiple sequential draws
- [x] Add turn-based automatic draw in `onEnterStep(Step.DRAW)`
- [x] Add `DRAW_CARD` action type for effect-triggered draws
- [x] Create comprehensive draw tests
- [x] Run tests, lint, format

#### Phase 4: State-Based Actions Extension ✅
- [x] Extend `StateBasedActions.ts` with `findPlayersWhoAttemptedEmptyDraw(game)`
- [x] Update `performStateBasedActions()` to check empty library draw
- [x] Add game loss mechanism for affected players
- [x] Clear "attempted draw from empty library" flag after SBA resolution
- [x] Create tests for SBA timing and game loss
- [x] Run tests, lint, format

#### Phase 5: GameStateExport Integration ✅
- [x] Update `GameStateExporter.ts` to export library zone
- [x] Library export shows card count (hidden information per MTG rules)
- [x] Update tests for game state export
- [x] Run full test suite: `bun test && bun run lint && bun run format`

**Blockers**: None

**Notes**:
- Follows existing zone entity pattern (Hand.ts, Graveyard.ts, Battlefield.ts)
- Library is ordered zone - top card drawn first
- Two draw paths: turn-based (automatic in DRAW step) and effect-triggered (DRAW_CARD action)
- Empty library handling follows MTG rules 121.4, 704.5b (attempt → flag → SBA → loss)
