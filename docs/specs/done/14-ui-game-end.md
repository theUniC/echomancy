# UI: Game End

## Goal

Detect and display game end condition with winner announcement.

## What We Get When Done

When a player's life reaches 0 (or other win condition), the game ends with clear winner announcement.

## Player Experience

1. Win condition triggered (life reaches 0)
2. Game pauses
3. Winner announcement overlay appears
4. Clear indication of who won and why

## Acceptance Criteria

- [ ] Game detects when player reaches 0 life
- [ ] Winner overlay/modal appears
- [ ] Shows who won and win condition
- [ ] Game stops accepting inputs after end

## Dependencies

- Combat damage (11) can reduce life to 0
- All core gameplay specs complete

## Out of Scope

- Deck-out loss condition
- Concede button
- Rematch option
- Match history
