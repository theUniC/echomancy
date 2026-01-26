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
- Game state export includes finished status
- Game state export includes outcome (winner/draw and reason)
- UI can display winner and reason

## Game Rules & Mechanics

### MVP Win/Lose Conditions (Backend)

**Life Total Loss** (104.3a):
- A player with 0 or less life loses the game
- Checked during state-based actions
- Multiple players can lose simultaneously

**Empty Library Loss** (104.3c):
- A player who attempts to draw from an empty library loses the game
- Tracking already implemented (`playersWhoAttemptedEmptyLibraryDraw`)
- Just needs to trigger game end instead of clearing the flag

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

**Current state**: ACTIVE (game in progress)
**New state**: FINISHED (game terminated)

**Outcome types**:
- WIN: One player wins, includes winner ID and reason (e.g., "LIFE_TOTAL")
- DRAW: No winner, includes reason (e.g., "SIMULTANEOUS_LOSS")

### Rejecting Actions in FINISHED State

**Rule**: Once a game is FINISHED, all action submissions are rejected.

**Behavior**:
- Any attempt to submit an action to a finished game should fail
- This includes: priority passing, spell casting, attacking, and all other game actions

### State Export in FINISHED State

**Rule**: Game state remains queryable after game ends.

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
- [x] Player with life ≤ 0 loses immediately during SBA check
- [x] Winning player is correctly identified
- [x] Game status transitions to FINISHED
- [x] Outcome includes winner ID and reason "LIFE_TOTAL"

**Empty Library Loss**:
- [x] Player who attempted to draw from empty library loses during SBA check
- [x] Existing tracking mechanism triggers game end
- [x] Outcome includes reason "EMPTY_LIBRARY"

**Simultaneous Loss (Draw)**:
- [x] Both players losing simultaneously results in DRAW
- [x] Outcome includes reason "SIMULTANEOUS_LOSS"
- [x] No winner is declared

**Action Rejection**:
- [x] Submitting actions to a finished game fails with clear error
- [x] All action types are rejected (pass priority, cast spell, etc.)

**State Export**:
- [x] Game state remains queryable after game ends
- [x] Export includes finished status
- [x] Export includes outcome with winner/draw information
- [x] Final board state is visible in export

## Out of Scope

**Deferred to future specs**:
- Poison counters (104.3d)
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
- Game state export (already implemented)

**Enables after implementation**:
- Spec 14: UI Game End Display
- Future: Match results API
- Future: Rematch system

