# UI Phase 2: Basic Actions

## Goal

Play lands, pass priority, end turn.

## What We Get When Done

Players can click lands to play them, click "Pass Priority" to pass, and click "End Turn" to end their turn. Error messages appear when illegal actions are attempted.

## Player Experience

- Click land in hand during main phase -> land moves to battlefield
- Click "Pass Priority" button -> priority passes to opponent
- Click "End Turn" button -> turn advances to opponent
- Attempting illegal actions shows clear error message
- See visual indication of who has priority

## Interactions

| Action | Trigger | Result |
|--------|---------|--------|
| Play land | Click land in hand | Land appears on battlefield |
| Pass priority | Click "Pass" button | Priority indicator moves to opponent |
| End turn | Click "End Turn" button | Turn counter increments, opponent's turn |

## Priority Indication

When you have priority:
- Your area is highlighted
- "Pass" and "End Turn" buttons are enabled
- Cards in hand are clickable

When opponent has priority:
- Opponent's area is highlighted
- Buttons are disabled
- Cards in hand are not clickable

## Acceptance Criteria

- [ ] Can play one land per turn during main phase
- [ ] Cannot play land at wrong time (error message shown)
- [ ] Cannot play more than one land per turn (error message shown)
- [ ] Can pass priority (button click)
- [ ] Can end turn (button click)
- [ ] Visual indicator shows who has priority
- [ ] Buttons disabled when not your priority

## Dependencies

- Phase 1 complete (visual display working)
  - Phase 1a: Route and data pipeline
  - Phase 1b: Basic game info display
  - Phase 1b.5: Starting hand bootstrap (cards in hand to play)
  - Phase 1c: Battlefield display
  - Phase 1d: Hand display

## Out of Scope

- Casting spells
- Targeting
- Mana payment
- Combat
