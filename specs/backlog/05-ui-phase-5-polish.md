# UI Phase 5: Polish

## Goal

Complete, playable MVP where two players can play a full game.

## What We Get When Done

A complete game experience: graveyard viewing, activated abilities, all error states handled gracefully, clear feedback for every action. Two players can play from start to finish on the same screen.

## Features to Complete

### Graveyard Viewing
- Click graveyard pile to expand and see all cards
- Both players can view either graveyard
- Click outside or "Close" to collapse

### Activated Abilities
- Click permanent with activated ability
- If ability needs targets, targeting flow starts
- If ability needs mana, mana payment flow
- Ability goes on stack

### Error Handling
- All illegal actions show clear, helpful error messages
- Errors disappear after a few seconds or on dismiss
- Never silent failures

### Visual Feedback
- Every action has visual confirmation
- Loading states if anything is slow
- Clear indication of game winner when game ends

## Acceptance Criteria

- [ ] Can play a complete game from start to finish
- [ ] Two players can take turns on the same screen
- [ ] Graveyard contents viewable
- [ ] Activated abilities work (tap for mana, other abilities)
- [ ] All error states show helpful messages
- [ ] Game end condition detected and winner announced
- [ ] All existing engine tests reproducible through UI
- [ ] No dead ends or stuck states in UI

## Definition of MVP Complete

The UI MVP is complete when:

1. Two players can play a full game on one screen
2. All card types work (creatures, lands, instants, sorceries)
3. Combat resolves correctly
4. Stack resolves in correct order (LIFO)
5. Priority passes correctly between players
6. Error messages are clear and helpful
7. A player can win by reducing opponent to 0 life

## Dependencies

- Phase 4 complete (combat working)

## Out of Scope (Future)

- Deck building
- Online multiplayer
- Player accounts / match history
- Card art/images (text only is acceptable)
- Animations or transitions
- Sound effects
- Mobile responsive design
- Undo functionality
- Timer/clock
- Drag and drop
- Keyboard shortcuts
