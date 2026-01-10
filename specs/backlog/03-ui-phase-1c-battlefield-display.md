# UI Phase 1c: Battlefield Display

## Goal

Display cards on the battlefield for both players, showing card names, tapped state, and creature stats.

## What We Get When Done

The `/games/[gameId]` page shows:
- Your battlefield permanents
- Opponent's battlefield permanents
- Tapped/untapped indication
- Creature power/toughness
- Basic card information

## Status

**Placeholder spec** - Will be fully detailed after Phase 1b is complete.

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- Phase 1b complete (basic game info displayed)

## Open Questions

- Card layout (simple list vs. structured boxes)
- Tapped indication (emoji, rotation, text)
- Grouping strategy (by type, by controller)
- How to display non-creature permanents (lands, artifacts, enchantments)

These questions will be answered during brainstorming after Phase 1b implementation.
