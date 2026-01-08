# Debug Game Management (Phase 0 Enhancement)

## Goal

Enable developers to list and resume existing games in the Debug Console, eliminating the need to create new games for every testing session.

## What We Get When Done

The Debug Console gains the ability to:
- Display all existing games with key metadata
- Select any game to load and continue working with it
- Create new games (existing functionality preserved)

This makes engine testing more efficient by allowing developers to return to games in specific states.

## Player Experience

### Listing Games
1. Developer opens the Debug Console
2. A "Load Existing Game" section shows all games with:
   - Game ID (truncated for readability)
   - Status (not started / in progress / finished)
   - Player names
   - Current turn number and phase (if started)
3. Games are sorted by most recently modified

### Selecting a Game
1. Developer clicks on a game from the list
2. The game loads into the console
3. Player IDs populate automatically
4. Current game state displays in the JSON viewer
5. Developer can immediately submit actions to this game

### Creating New Games
- Works exactly as before (unchanged)
- New games appear in the list after creation

## Acceptance Criteria

### Backend
- [ ] `GET /api/games` returns list of all games
- [ ] Response includes: gameId, status, playerNames, turnNumber, currentPhase
- [ ] Follows existing API conventions (data wrapper, error format)

### Frontend
- [ ] Debug Console shows list of existing games
- [ ] Clicking a game loads it (populates IDs, fetches state)
- [ ] Selected game is visually indicated
- [ ] Can switch between games without page reload
- [ ] Creating a new game adds it to the list
- [ ] Error handling for failed game loads

## Dependencies

- Phase 0 Debug Console complete (done)
- `GET /api/games/[gameId]/state` endpoint exists (done)

## Out of Scope

- Game deletion
- Game filtering or search
- Pagination (assume small number of test games)
- Real-time updates (manual refresh is fine)
- Persistent storage (in-memory repository is acceptable)
