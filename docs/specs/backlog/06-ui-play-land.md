# UI: Play Land

## Goal

Allow players to play land cards from hand to battlefield.

## What We Get When Done

Players can click a land in their hand during their main phase to play it. The land moves to the battlefield. Error messages appear for illegal plays.

## Player Experience

1. During your main phase, lands in hand are visually indicated as playable
2. Click a land card in hand
3. Land moves to battlefield
4. If illegal (wrong phase, already played land this turn), error message shown

## Acceptance Criteria

- [ ] Clicking land in hand during main phase plays it to battlefield
- [ ] Cannot play land during wrong phase (error message)
- [ ] Cannot play more than one land per turn (error message)
- [ ] Visual indication of playable lands during main phase

## Dependencies

- Phase 1 complete (all display specs: 1a through 1e)
- Hand display showing clickable cards

## Out of Scope

- Pass priority (separate spec)
- End turn (separate spec)
- Casting spells
