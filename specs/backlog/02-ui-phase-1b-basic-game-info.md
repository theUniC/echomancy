# UI Phase 1b: Basic Game Info Display

## Goal

Display core game state information (turn, phase, life totals) on the game page, replacing the "snapshot loaded" placeholder with actual data.

## What We Get When Done

The `/games/[gameId]` page shows:
- Turn number
- Current phase and step
- Player 1's life total
- Opponent's life total

Simple text display, no styling beyond basic layout.

## Player Experience

User navigates to `/games/{gameId}` and sees:

```
Turn 3 - Main Phase 1

Your Life: 20
Opponent Life: 18
```

Clean, readable game state at a glance.

## Implementation Steps

### Step 5: Display Turn Number
- Read `snapshot.publicGameState.turnNumber`
- Display: "Turn {number}"

### Step 6: Display Current Phase
- Read `snapshot.publicGameState.currentPhase` and `currentStep`
- Display: "Main Phase 1", "Combat - Declare Attackers", etc.
- Format step name to be human-readable (convert DECLARE_ATTACKERS → "Declare Attackers")

### Step 7: Display Player 1's Life Total
- Read `snapshot.privatePlayerState.life`
- Display: "Your Life: {life}"

### Step 8: Display Opponent's Life Total
- Read `snapshot.opponentStates[0].life`
- Display: "Opponent Life: {life}"
- Handle case where there are multiple opponents (show first one for MVP)

## Acceptance Criteria

### Turn Display
- [ ] Turn number is visible
- [ ] Updates correctly as game progresses (can verify in debug console)

### Phase Display
- [ ] Current phase name is visible (e.g., "Main Phase 1")
- [ ] Current step is visible if applicable (e.g., "Declare Attackers")
- [ ] Step names are human-readable (not SCREAMING_SNAKE_CASE)

### Life Totals
- [ ] Player 1's life is displayed
- [ ] Opponent's life is displayed
- [ ] Values match the actual game state (verify against raw JSON in debug)

### Layout
- [ ] Information is organized and readable
- [ ] Clear visual separation between sections
- [ ] No overlapping text

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- `GET /api/games/[gameId]/state` returns valid data (done)

## Out of Scope

- Priority indicator (who has priority)
- Mana pool display
- Any zone display (hand, battlefield, graveyard)
- Stack display
- Styling/visual polish
- Multiple opponents (only show first opponent)

## Testing the Feature

To test this feature manually:
1. Create a game in `/debug`
2. Submit some actions (advance turn, play land, etc.)
3. Navigate to `/games/{game-id}`
4. Verify turn number, phase, and life totals match debug console JSON

## Technical Notes

- Phase/step formatting: Convert "MAIN_1" → "Main Phase 1", "DECLARE_ATTACKERS" → "Declare Attackers"
- Use helper function for phase/step name formatting
- For multiple opponents, access `opponentStates[0]` (MVP assumes 2-player games)
- All data comes from GameSnapshot (already created in Phase 1a)

## Future Work (Not in This Phase)

- Priority indicator
- Active player indicator
- Mana pool display
- Player names (currently just "Your" and "Opponent")
- Support for >2 players
