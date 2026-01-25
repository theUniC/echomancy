# Game Setup: Deck Loading and Opening Hand

## Overview

Replace the current empty starting state with a real game setup flow: load decks → shuffle → draw 7 cards. Each player begins with a randomized hand from their deck.

## Design Goals

- Load deck lists and create card objects in library zones
- Shuffle libraries randomly
- Draw 7-card opening hands
- Keep Game engine agnostic to card source (extensible for future)

## Card Source Strategy

**MVP (now):** Mini-catalog of card definitions in code.

**Future (extensible):** External data source (JSON, DB, API).

The Game receives card definitions and doesn't care where they come from. The interface is stable; only the source changes.

### Basic Cards (MVP)

Create ~10 generic cards for testing:

| Card | Type | Stats |
|------|------|-------|
| Forest | Land | - |
| Mountain | Land | - |
| Plains | Land | - |
| Island | Land | - |
| Swamp | Land | - |
| Bear | Creature | 2/2 |
| Elite Vanguard | Creature | 2/1 |
| Giant Growth | Instant | - |
| Lightning Strike | Instant | - |
| Divination | Sorcery | - |

These are placeholder cards. Real card effects are out of scope.

### Prebuilt Decks (MVP)

Two simple 60-card decks for testing:

**Green Deck:**
- 24x Forest
- 20x Bear (2/2 creature)
- 16x Giant Growth (instant)

**Red Deck:**
- 24x Mountain
- 20x Elite Vanguard (2/1 creature)
- 16x Lightning Strike (instant)

## Game Rules

Per MTG Comprehensive Rules:
- **103.2**: Minimum 60 cards (constructed)
- **103.4**: Draw 7 cards for opening hand
- **103.5**: Mulligan (out of scope for MVP)

## Acceptance Criteria

### Basic Cards
- [x] ~10 card definitions available (5 lands, 2 creatures, 3 spells)
- [x] Cards have valid types, names, and stats
- [x] Prebuilt Green and Red deck configurations available

### Deck Loading
- [x] Game can be started with optional deck configurations
- [x] Each card in the deck becomes a unique game object
- [x] Cards are placed in player's library zone

### Shuffle
- [x] Library is shuffled after loading
- [x] Shuffle is random in production
- [x] Shuffle can be deterministic for testing

### Opening Hand
- [x] Each player draws 7 cards after shuffle
- [x] Cards move from LIBRARY to HAND
- [x] Library shows 53 cards after draw

### Backward Compatibility
- [x] Starting a game without decks still works (empty libraries)
- [x] Existing tests pass without modification

## Out of Scope

- Mulligan system
- Deck validation (60-card minimum, 4-of limit)
- Deck builder UI
- Custom deck selection
- Card effects (cards are mechanical placeholders)

## Implementation Tracking

**Status**: Done
**Started**: 2026-01-25
**Completed**: 2026-01-25
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Basic Cards Catalog ✅
- [x] Create ~10 card definitions (5 lands, 2 creatures, 3 spells)
- [x] Create prebuilt Green and Red deck configurations

#### Phase 2: Deck Loading ✅
- [x] Convert card definitions to unique game objects
- [x] Random shuffle with testable seeding
- [x] Unit tests

#### Phase 3: Game Integration ✅
- [x] Game start accepts optional deck configurations
- [x] Load decks, shuffle, draw 7 cards
- [x] Backward compatibility (start without decks)
- [x] Integration tests

#### Phase 4: QA and Finalization ✅
- [x] All Acceptance Criteria verified
- [x] All tests pass
- [x] Lint passes
- [x] Code review
- [x] Move spec to `done/`

**Blockers**: None
**Notes**: Backend-only. No UI changes. Cards are mechanical placeholders without real effects.
