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

- [x] Clicking land in hand during main phase plays it to battlefield
- [x] Cannot play land during wrong phase (error message)
- [x] Cannot play more than one land per turn (error message)
- [x] Visual indication of playable lands during main phase

## Dependencies

- Phase 1 complete (all display specs: 1a through 1e)
- Hand display showing clickable cards

## Out of Scope

- Pass priority (separate spec)
- End turn (separate spec)
- Casting spells

---

## Implementation Tracking

**Status**: In Progress
**Started**: 2026-01-17
**Completed**:
**Agent**: ui-engineer (Phase 2)

### Task Breakdown

#### Phase 1: Backend - Allowed Actions Query (senior-backend-engineer) ✅

- [x] Create `GetAllowedActionsQuery` class in `src/echomancy/application/query/get-allowed-actions/`
- [x] Create `GetAllowedActionsQueryHandler` with player ID validation
- [x] Create API route `GET /api/games/[gameId]/allowed-actions` with playerId query param
- [x] Add tests for query handler (various game states)
- [x] Add tests for API route (happy path, errors)
- [x] Run `bun test && bun run lint && bun run format`

#### Phase 2: Frontend - Click Handlers and API Integration (ui-engineer) ✅

- [x] Add `onCardClick?: (cardId: string) => void` prop to `CardSprite`
- [x] Configure PixiJS sprite for interactivity (`eventMode`, cursor)
- [x] Add click handler propagation through `HandZone`
- [x] Create `usePlayLand` hook or function for API call to `/api/games/[gameId]/actions`
- [x] Implement state refresh after successful action
- [x] Add error handling and display (simple alert/toast for errors)
- [x] Run `bun test && bun run lint && bun run format`

#### Phase 3: Frontend - Playable Card Indication (ui-engineer) ✅

- [x] Fetch allowed actions on game state load
- [x] Compute `playableCardIds` from allowed actions + card types
- [x] Add `isPlayable?: boolean` prop to `CardSprite`
- [x] Add visual effect (glow/highlight) for playable cards
- [x] Ensure highlight updates after playing land or phase change
- [x] Run `bun test && bun run lint && bun run format`

#### Phase 4: Integration Testing and Polish (mtg-code-reviewer + ui-engineer)

- [ ] Code review all changes
- [ ] Verify all acceptance criteria manually
- [ ] Fix any issues found in review
- [ ] Final `bun test && bun run lint && bun run format`

**Blockers**: None

**Notes**:
- Phases 1 and 2 can run in parallel (backend + frontend click handlers)
- Phase 3 depends on Phase 1 completion (needs allowed actions API)
- Backend PLAY_LAND action already fully implemented in game engine
- API endpoint `/api/games/[gameId]/actions` already handles PLAY_LAND
- Phase 1 complete: Query, handler, API endpoint, and tests all implemented
- Phase 2 complete: All UI changes implemented, tests passing, code formatted
- Phase 3 complete: Visual indication of playable lands fully implemented
  - Allowed actions fetched on page load and after each action (page.tsx lines 85-95)
  - playableCardIds passed to HandZone via BattlefieldDisplay (BattlefieldDisplay.tsx line 145)
  - isPlayable prop added to CardSprite (CardSprite.tsx line 39)
  - Green border (4px) rendered for playable cards (CardSprite.tsx lines 69-88)
  - All tests passing (432 pass, 0 fail)
