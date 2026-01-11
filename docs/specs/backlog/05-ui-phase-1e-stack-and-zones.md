# UI Phase 1e: Stack & Additional Zones

## Goal

Display the stack (when not empty) and graveyard indicators, completing the read-only game view.

## What We Get When Done

The `/games/[gameId]` page shows:
- Stack contents (spells and abilities waiting to resolve)
- Graveyard card counts
- Complete read-only view of all relevant zones

## Status

**Placeholder spec** - Will be fully detailed after Phase 1d is complete.

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- Phase 1b complete (basic game info displayed)
- Phase 1c complete (battlefield display)
- Phase 1d complete (hand display)

## Open Questions

- Stack display format (top-to-bottom, list, compact)
- How to show stack targets
- Graveyard display (count only, last card, viewable list)
- Whether to show exile zone (currently not implemented in engine)

These questions will be answered during brainstorming after Phase 1d implementation.
