# UI Phase 1: Incremental Breakdown Design

**Date**: 2026-01-10
**Status**: Approved
**Author**: Brainstorming session with Claude Code

## Overview

UI Phase 1 (Read-Only Display) has been broken down into ultra-granular increments to reduce uncertainty and enable rapid iteration. Each increment builds on the previous one with minimal complexity added per step.

## Problem Statement

The original "UI Phase 1: Read-Only Display" spec attempted to implement all zones, card states, turn info, and stack display in one feature. This introduced significant uncertainty around:
- Card rendering approach (text vs. boxes vs. styled components)
- Layout decisions (single column, two column, MTG Arena-style)
- Error handling patterns
- GameSnapshot integration details

## Design Decision: Ultra-Granular Phases

We split Phase 1 into five sub-phases, each with 4-8 steps that can be completed in minutes:

### Phase 1a: Route & Data Pipeline (4 steps)
**Goal**: Validate the entire data flow works with zero UI complexity

Steps:
1. Create route that displays "Game Page"
2. Extract and display gameId from URL params
3. Fetch game state with loading/error states
4. Create GameSnapshot and display success message

**Key decisions**:
- URL pattern: `/games/[gameId]` (simplified from `/games/[gameId]/players/[playerId]`)
- Always use Player 1's perspective (hardcoded for Phase 1)
- Error states shown inline on the game page
- Static fetch on load (no polling)

### Phase 1b: Basic Game Info (4 steps)
**Goal**: Display core game state information

Steps:
5. Display turn number
6. Display current phase and step (human-readable)
7. Display Player 1's life total
8. Display opponent's life total

**Key decisions**:
- Classic MTG Arena layout (opponent top, you bottom)
- Simple text display (no styling beyond basic layout)
- Phase names converted to human-readable format

### Phase 1c: Battlefield Display (TBD)
**Goal**: Show cards on battlefield for both players

**Status**: Placeholder spec - will be detailed after 1b is complete

Open questions to answer during next brainstorming:
- Card rendering approach (text list, boxes, structured)
- Tapped indication method
- Creature stat display
- Non-creature permanent display

### Phase 1d: Hand Display (TBD)
**Goal**: Show viewer's hand cards and opponent hand count

**Status**: Placeholder spec - will be detailed after 1c is complete

### Phase 1e: Stack & Additional Zones (TBD)
**Goal**: Show stack and graveyard, completing read-only view

**Status**: Placeholder spec - will be detailed after 1d is complete

## Architecture

### URL Structure
```
/games/[gameId]
```

Simple, RESTful, supports deep linking.

Future enhancement: Add `/games/[gameId]/players/[playerId]` for multi-player perspective switching.

### Data Flow
```
Page loads
  ↓
Extract gameId from URL params
  ↓
Fetch GET /api/games/[gameId]/state
  ↓
Create GameSnapshot for Player 1 (turnOrder[0])
  ↓
Render snapshot data
```

### Component Structure
```
/src/app/games/[gameId]/page.tsx  (Client Component)
  - Fetches game state
  - Creates GameSnapshot
  - Renders game info
  - Handles loading/error states
```

### Rendering Technology Strategy

**Phase 1a-1b: HTML/CSS/React**
- Simple text display (loading states, game info, life totals)
- Standard React patterns
- No special rendering libraries

**Phase 1c-1e: PixiJS + HTML/CSS Hybrid**
- **PixiJS for cards**: Battlefield cards, hand cards, all card rendering
- **HTML/CSS for UI chrome**: Turn/phase display, life totals, buttons, static info
- **Why this split**: PixiJS provides performance and future flexibility for card interactions (drag & drop, animations), while HTML/CSS remains simpler for standard UI elements

**PixiJS Introduction Point**: Phase 1c (Battlefield Display)

This decision avoids premature complexity in Phase 1a-1b while establishing the rendering foundation before Phase 2 (interactions).

### Error Handling
- Game not found (404) → "Error: Game not found"
- Network error → "Error: Failed to load game"
- No players in game → "Error: No players in game"
- All errors shown inline with helpful links to Debug Console

## Implementation Notes

### Technologies
- Next.js 16 App Router (Client Component)
- React 19 (useState, useEffect)
- GameSnapshot from `infrastructure/ui`
- Existing API endpoint: `GET /api/games/[gameId]/state`

### Card Registry
Phase 1a requires a CardRegistry to resolve card definition IDs to names. Use a simple in-memory registry:

```typescript
const cardRegistry = {
  getCardName: (id: string) => cardDefinitions[id]?.name || "Unknown Card"
}
```

Future work: Load from database or centralized registry.

### Phase/Step Formatting
Convert engine format to human-readable:
- `MAIN_1` → "Main Phase 1"
- `DECLARE_ATTACKERS` → "Declare Attackers"

Use a simple helper function for this transformation.

## Open Questions

### Deferred to Future Phases
- Player perspective switching (Phase 2+)
- Real-time updates / polling (Phase 2+)
- Card visual styling (Phase 5 - Polish)
- Navigation to/from game page (Phase 2+)

### To Be Answered in Phase 1c Brainstorming
- Card rendering approach
- Tapped state visualization
- Battlefield layout with many cards

## Success Criteria

**Phase 1a is successful when**:
- User can navigate to `/games/{valid-game-id}`
- Page shows "Game snapshot loaded successfully for Player 1"
- Invalid game IDs show appropriate error messages
- No crashes or console errors

**Phase 1b is successful when**:
- Turn number, phase, and both life totals are visible
- Information matches the actual game state
- Phase names are human-readable

## Rationale

### Why Ultra-Granular?
- Reduces uncertainty per step
- Each step takes minutes, not hours
- Can stop anytime and have working code
- Easier to review and test
- Builds confidence incrementally

### Why Static Fetch (No Polling)?
- Phase 1 is "read-only display"
- Action submission in Phase 2 will naturally refresh state
- Keeps scope minimal
- Can add polling later if needed

### Why Player 1 Hardcoded?
- Simplifies Phase 1 significantly
- No need for player selection UI
- Can add player switching when we add interactions
- URL structure supports it when needed (`/players/[playerId]`)

## Future Enhancements

- Player perspective selection
- Auto-refresh / polling
- Multiple opponent support
- Spectator mode (full visibility)
- URL includes player ID for deep linking
- Card hover previews
- Visual styling and polish

## Maintenance

This design document should be updated when:
- New phases are planned with detailed specs
- Architecture decisions change
- New open questions emerge during implementation

## Summary

UI Phase 1 has been broken into 5 sub-phases with ultra-granular steps. This enables rapid, low-risk iteration with minimal uncertainty per step. The first phase (1a) validates the entire data pipeline, while subsequent phases incrementally add display elements.
