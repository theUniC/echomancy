# UI Phase 1a: Route & Data Pipeline

## Goal

Establish the data pipeline from URL to GameSnapshot with minimal UI, validating that routing, API integration, and GameSnapshot creation all work correctly.

## What We Get When Done

A page at `/games/[gameId]` that:
- Accepts a game ID in the URL
- Fetches the game state from the API
- Creates a GameSnapshot for Player 1's perspective
- Displays confirmation that the snapshot was created successfully

This validates the entire data flow without any complex rendering.

## Player Experience

### Accessing a Game
1. User navigates to `/games/abc-123-xyz` (any valid game ID)
2. Page loads and shows "Loading game..."
3. Page fetches game state and creates snapshot
4. Page shows "Game snapshot loaded successfully for Player 1"

### Error Cases
1. Invalid game ID → Shows "Error: Game not found"
2. Network error → Shows "Error: Failed to load game"
3. Game has no players → Shows "Error: No players in game"

## Implementation Steps

### Step 1: Create Basic Route
- Create `src/app/games/[gameId]/page.tsx`
- Display static text: "Game Page"
- No data fetching yet

### Step 2: Display Game ID
- Extract `gameId` from URL params
- Display it on the page: "Game ID: {gameId}"

### Step 3: Fetch Game State with Error Handling
- Call `GET /api/games/[gameId]/state` on component mount
- Show loading state while fetching
- Show error state if fetch fails
- Show success state when data arrives

### Step 4: Create GameSnapshot
- Import `createGameSnapshot` from infrastructure/ui
- Get Player 1 ID from `turnOrder[0]`
- Create snapshot for Player 1's perspective
- Display: "Game snapshot loaded successfully for Player 1"

## Acceptance Criteria

### Routing
- [ ] `/games/[gameId]` route exists and renders
- [ ] Game ID is extracted from URL params correctly

### Data Fetching
- [ ] Fetches game state from `/api/games/[gameId]/state` on mount
- [ ] Shows "Loading game..." while fetching
- [ ] Shows error message if game doesn't exist
- [ ] Shows error message if network request fails

### GameSnapshot Creation
- [ ] Creates GameSnapshot using Player 1 ID (from turnOrder[0])
- [ ] Handles games with no players (shows error)
- [ ] Displays success confirmation when snapshot is created

### Error States
- [ ] "Game not found" shown for invalid game ID (404)
- [ ] "Failed to load game" shown for network errors
- [ ] "No players in game" shown when turnOrder is empty

## Dependencies

- `GET /api/games/[gameId]/state` endpoint exists (done)
- `GameSnapshot.createGameSnapshot()` exists (done)
- Debug Console can create games with players (done)

## Out of Scope

- Displaying any game state information (turn, phase, life, zones)
- Player selection (always uses Player 1)
- Refresh/polling functionality
- Navigation to/from game page
- Any visual styling beyond basic layout

## Testing the Feature

To test this feature manually:
1. Go to `/debug` and create a new game
2. Note the game ID displayed
3. Navigate to `/games/{that-game-id}`
4. Verify success message appears
5. Try invalid game ID → verify error message

## Technical Notes

- This is a Client Component (`"use client"`)
- Uses React useState for loading/error/snapshot state
- Uses useEffect for data fetching on mount
- GameSnapshot requires a CardRegistry - use a simple in-memory registry for now
- Error messages should be user-friendly, not technical

## Future Work (Not in This Phase)

- Actual game state display (Phase 1b)
- Player perspective switching (Phase 2+)
- Real-time updates (Phase 2+)
- Card registry from database (future)
