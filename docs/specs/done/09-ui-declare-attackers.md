# UI: Declare Attackers

## Goal

Allow active player to select creatures as attackers during combat.

## What We Get When Done

During declare attackers step, player can click creatures to toggle attack selection, then confirm to proceed.

## Player Experience

1. Enter combat phase (automatic when passing priority in first main)
2. See "Declare Attackers" indicator
3. Click untapped creatures to toggle attack selection
4. Selected attackers show visual indicator (red border, sword icon)
5. Click "Confirm Attackers" to proceed

## Acceptance Criteria

- [ ] Can select multiple attackers by clicking
- [ ] Can deselect attacker by clicking again
- [ ] Tapped creatures cannot be selected
- [ ] Confirm button proceeds to blockers step

## Dependencies

- Priority controls (07) working
- Spell casting (08) establishes targeting patterns

## Out of Scope

- Blocker declaration (separate spec)
- Combat damage (separate spec)
- Combat keywords (flying, etc.)
