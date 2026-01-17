# B1-03: Backend Win/Lose Conditions

## Overview

**Feature**: Core game termination rules for MTG win/lose conditions.

**Design Goals**:
- Detect game-ending conditions via state-based actions
- Track game outcome (winner/loser/draw)
- Transition game to FINISHED state when conditions are met
- Support MVP win conditions (life totals only)

**Relationship to Other Systems**:
- Depends on: State-based actions system, player life tracking
- Enables: Game end UI (spec 14), match results, rematch flow
- Part of: Core MTG rules compliance

## User Stories

**As a player**, I want the game to automatically detect when I've won or lost, so I don't have to manually declare the outcome.

**As a player**, I want to see who won and why, so I understand the game result.

**As a developer**, I want game termination to be a rules-enforced state transition, so the UI never has to decide when a game is over.

## Player Experience

**What the player sees**:
1. Player A's life total reaches 0 during state-based action check
2. Game state transitions to FINISHED
3. UI displays "Player B wins! (Player A lost due to having 0 or less life)"
4. No further game actions are possible
5. Players can view final board state

**Player flow**:
- During any game action, if state-based actions run and detect a win/lose condition
- Game immediately transitions to FINISHED state
- Priority passing, action submission, and turn progression all stop
- Game state becomes read-only (via UI layer)
- Final game state remains queryable for UI display

**Feedback mechanisms**:
- `GameExport` includes `status: "FINISHED"`
- `GameExport` includes `outcome: { type: "WIN", winner: PlayerId, reason: "LIFE_TOTAL" }`
- UI can display winner and reason

## Game Rules & Mechanics

### MVP Win/Lose Conditions (Backend)

**Life Total Loss** (104.3a):
- A player with 0 or less life loses the game
- Checked during state-based actions
- Multiple players can lose simultaneously

**No Players Remaining** (104.3):
- If all players lose simultaneously, the game is a draw
- Example: Both players at 0 life from simultaneous damage

### State-Based Action Integration

**When checked**:
- After any action resolves
- Before priority is passed
- During stack resolution between objects

**Detection flow**:
1. State-based actions check all players' life totals
2. If any player has life ≤ 0, that player loses
3. If all players lose, game is a draw
4. If only one player remains, that player wins
5. Game transitions to FINISHED state with outcome recorded

### Game State Transition

**Current state**: `ACTIVE` (game in progress)
**New state**: `FINISHED` (game terminated)

**Status values**:
```typescript
type GameStatus = "ACTIVE" | "FINISHED"
```

**Outcome structure**:
```typescript
type GameOutcome =
  | { type: "WIN"; winner: PlayerId; reason: WinReason }
  | { type: "DRAW"; reason: DrawReason }
  | null // Game not finished

type WinReason = "LIFE_TOTAL" // MVP only
type DrawReason = "SIMULTANEOUS_LOSS" // MVP only
```

### Rejecting Actions in FINISHED State

**Rule**: Once a game is FINISHED, all action submissions are rejected.

**Implementation requirement**:
- `game.submitAction()` checks game status
- If status is FINISHED, throw error: "Cannot submit actions to a finished game"
- This prevents:
  - Priority passing
  - Spell casting
  - Attacking
  - Any other game actions

### State Export in FINISHED State

**Rule**: `game.exportState()` continues to work after game ends.

**Why**: The UI needs to display:
- Final board state
- Final life totals
- Winner/loser information
- Game history

## Edge Cases

### Both Players Lose Simultaneously
**Situation**: Both players at 1 life, a spell deals 1 damage to each player
**Expected**: Game is a DRAW, reason "SIMULTANEOUS_LOSS"
**Test**: Both players lose during same SBA check

### Player Loses During Opponent's Turn
**Situation**: Player A's life reaches 0 during Player B's turn
**Expected**: Player B wins immediately, game transitions to FINISHED
**Test**: Life loss during opponent's turn ends game correctly

### Player Loses During Stack Resolution
**Situation**: Multiple spells on stack, first spell causes life loss
**Expected**: Stack resolution stops, SBA check runs, game ends
**Test**: Game ends mid-stack, remaining stack items are not resolved

### Last Surviving Player Wins
**Situation**: In future multiplayer, only 1 player remains after multiple losses
**Expected**: Last player wins
**Deferred**: Multiplayer support is post-MVP

## Acceptance Criteria

**Life Total Loss**:
- [ ] Player with life ≤ 0 loses immediately during SBA check
- [ ] Winning player is correctly identified
- [ ] Game status transitions to FINISHED
- [ ] Outcome includes winner ID and reason "LIFE_TOTAL"

**Simultaneous Loss (Draw)**:
- [ ] Both players losing simultaneously results in DRAW
- [ ] Outcome includes reason "SIMULTANEOUS_LOSS"
- [ ] No winner is declared

**Action Rejection**:
- [ ] `game.submitAction()` throws error if status is FINISHED
- [ ] Error message is clear: "Cannot submit actions to a finished game"
- [ ] All action types are rejected (pass priority, cast spell, etc.)

**State Export**:
- [ ] `game.exportState()` works after game ends
- [ ] Export includes `status: "FINISHED"`
- [ ] Export includes `outcome` with winner/draw information
- [ ] Final board state is visible in export

**Test Coverage**:
- [ ] Unit test: Player loses due to life total
- [ ] Unit test: Both players lose simultaneously (draw)
- [ ] Unit test: Actions rejected after game ends
- [ ] Unit test: State export works in FINISHED state
- [ ] Integration test: Full game ending scenario

## Out of Scope

**Deferred to future specs**:
- Poison counters (104.3d)
- Drawing from empty library (104.3c)
- Ten poison counters (104.3d)
- Conceding (104.3a)
- Commander damage (903.10a)
- Mill-based loss conditions
- Alternate win conditions (e.g., "you win the game" effects)
- Multiplayer rules (only 2-player supported in MVP)

**UI responsibilities** (see spec 14):
- Displaying winner/loser information
- Game end modal/overlay
- Rematch flow
- Match history

## Dependencies

**Required before implementation**:
- State-based actions system (already implemented)
- Player life tracking (already implemented)
- `GameExport` structure (already implemented)

**Enables after implementation**:
- Spec 14: UI Game End Display
- Future: Match results API
- Future: Rematch system

## Technical Notes for Implementation Team

**Recommended location**: `src/echomancy/core/game/Game.ts`

**Key implementation points**:
1. Add `status: GameStatus` field to Game class
2. Add `outcome: GameOutcome | null` field to Game class
3. In state-based actions logic, check life totals and set outcome
4. In `submitAction()`, check status and reject if FINISHED
5. In `exportState()`, include status and outcome
6. Write tests for all win/lose scenarios

**No HOW details**: Implementation team decides exact code structure, method names, and internal architecture.
