# UI: Graveyard Count

## Overview

Display graveyard card count for both players. Read-only indicator only.

## User Stories

**As a player**, I want to:
- See how many cards are in my graveyard
- See how many cards are in my opponent's graveyard

## Player Experience

- Each player's area shows a graveyard indicator with card count
- Format: "Graveyard: X" or similar compact display
- Visible at a glance without interaction

## Game Rules & Mechanics

- Graveyard is public information (both players can see counts)
- Count updates when cards go to graveyard (creature death, discards, etc.)

## Acceptance Criteria

- [x] Your graveyard count is displayed
- [x] Opponent graveyard count is displayed
- [x] Counts are accurate per GameSnapshot
- [x] Empty graveyard shows "0"

## Out of Scope

- Viewing individual cards in graveyard (see spec 12)
- Graveyard interactions
- Exile zone (see spec 16)
- Stack display (see spec 15)

## Dependencies

- Phase 1d complete (hand display)

---

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-17
**Completed**: 2026-01-17
**Agent**: ui-engineer

### Task Breakdown

#### Phase 1: Create GraveyardCount Component ✅
- [x] Create `src/app/games/[gameId]/components/graveyard/GraveyardCount.tsx`
- [x] Props: `count: number`, `label?: string` (default "Graveyard")
- [x] Match styling from `OpponentHandCount` (bg #0D1117, text #B0B0B0, Inter 16px semi-bold)
- [x] Handle singular/plural: "1 card" vs "X cards"

#### Phase 2: Integrate into GamePage ✅
- [x] Import `GraveyardCount` into `page.tsx`
- [x] Add player graveyard: `<GraveyardCount count={privatePlayerState.graveyard.length} label="Your Graveyard" />`
- [x] Add opponent graveyard: `<GraveyardCount count={opponentStates[0]?.graveyard.length ?? 0} label="Opponent Graveyard" />`

#### Phase 3: Quality Assurance ✅
- [x] Run `bun test` - all tests pass
- [x] Run `bun run lint && bun run format` - code style compliant

**Blockers**: None
**Notes**:
- Data already available in GameSnapshot (graveyard arrays exist for both player and opponent)
- Follow exact pattern from `OpponentHandCount.tsx`
- Component location: `src/app/games/[gameId]/components/graveyard/GraveyardCount.tsx`
