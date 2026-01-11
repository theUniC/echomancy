# UI Phase 1d: Hand Display

## Goal

Display hand cards for the viewing player (full visibility) and opponent hand count (hidden cards).

## What We Get When Done

The `/games/[gameId]` page shows:
- Your hand cards (names visible)
- Opponent's hand count (e.g., "Hand: 5 cards")

## Status

**Placeholder spec** - Will be fully detailed after Phase 1c is complete.

## Rendering Technology

**Uses PixiJS** (introduced in Phase 1c) for card rendering.

Hand cards will be rendered as PixiJS sprites/containers, continuing the pattern established in Phase 1c for battlefield cards. This ensures visual consistency and enables future features like drag & drop to play cards.

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- Phase 1b complete (basic game info displayed)
- Phase 1c complete (battlefield display with PixiJS)
- PixiJS integration established (from Phase 1c)

## Open Questions

- Hand card layout in PixiJS (fan arrangement, spacing, perspective)
- Card sorting (by type, cost, name)
- How to show card details without clicking (hover states, zoom)
- Card selection/highlight patterns for future interactions

These questions will be answered during brainstorming after Phase 1c implementation.
