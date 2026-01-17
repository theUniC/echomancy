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

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-17
**Completed**: 2026-01-17
**Agent**: ui-engineer

### Task Breakdown

#### Phase 1: Priority Indicator Component ✅
- [x] Create PriorityIndicator component in src/app/games/[gameId]/components/priority/
- [x] Display "Your Priority" or "Opponent's Priority" based on publicGameState.priorityPlayerId
- [x] Style with clear visual distinction (colors/borders) for both states
- [x] Add to GameInfo component in page.tsx

#### Phase 2: Priority Control Buttons ✅
- [x] Add "Pass" button that calls ADVANCE_STEP action (same pattern as handleCardClick)
- [x] Add "End Turn" button that calls END_TURN action
- [x] Disable both buttons when uiHints.canPassPriority is false
- [x] Use existing actionError state for error handling
- [x] Refresh game state after successful actions

#### Phase 3: Integration and Testing ✅
- [x] Verify priority indicator updates after actions
- [x] Verify buttons enable/disable correctly based on priority
- [x] Verify error messages display for failed actions
- [x] Run bun test and ensure all tests pass
- [x] Run bun run lint && bun run format

**Blockers**: None
**Notes**:
- All backend support already exists (ADVANCE_STEP, END_TURN actions)
- GameSnapshot already exposes priorityPlayerId and canPassPriority
- Follow Play Land action pattern from handleCardClick in page.tsx
- Priority indicator shows green "Your Priority" when viewer has priority, gray "Opponent's Priority" otherwise
- Both buttons are disabled when canPassPriority is false
- Error handling integrated with existing actionError state and displays errors inline
