# UI Phase 0: Debug Console

## Goal

Validate engine integration without building real UI.

## What We Get When Done

A simple textarea where you can paste JSON actions and see the resulting game state. This proves the engine works before investing in UI.

## Player Experience

1. Page shows a textarea and a "Submit" button
2. Paste a JSON action (e.g., `{"type": "PLAY_LAND", "playerId": "p1", "cardId": "forest-1"}`)
3. Click Submit
4. See the raw game state JSON below
5. See any errors if the action was invalid

## Acceptance Criteria

- [ ] Can create a new game with two players
- [ ] Can submit any valid action as JSON
- [ ] Can see the full game state after each action
- [ ] Shows clear error messages for invalid actions
- [ ] Game state persists between actions (same game session)

## Dependencies

None. This is the first phase.

## Out of Scope

- Visual representation of cards
- Click interactions
- Any styling beyond basic readability
