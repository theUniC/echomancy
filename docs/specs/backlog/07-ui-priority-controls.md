# UI: Priority Controls

## Goal

Allow players to pass priority and end their turn with clear visual feedback.

## What We Get When Done

Players see who has priority. When it's their turn, they can click "Pass" to pass priority or "End Turn" to end their turn. Buttons are disabled when not their priority.

## Player Experience

When you have priority:
- Your area is highlighted
- "Pass" and "End Turn" buttons are enabled
- Interactive elements are clickable

When opponent has priority:
- Opponent's area is highlighted
- Buttons are disabled
- Waiting state is clear

## Acceptance Criteria

- [ ] Visual indicator shows who has priority
- [ ] "Pass" button passes priority to opponent
- [ ] "End Turn" button ends turn and passes to opponent
- [ ] Buttons disabled when not your priority

## Dependencies

- Phase 1 complete (all display specs)
- Play land spec (06) - establishes clickable card pattern

## Out of Scope

- Playing lands (separate spec)
- Casting spells
- Combat phase controls
