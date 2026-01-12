# UI Phase 1b.5: Starting Hand Bootstrap

## Goal

Players start with a predetermined hand of 7 cards when a game begins, enabling the UI phases that follow (battlefield display, hand display) to have real card data to display.

## Why This Phase Exists

Phases 1c (Battlefield Display) and 1d (Hand Display) need cards to display. Without this bootstrap:
- The UI would show empty zones
- No meaningful testing of card rendering is possible
- UI development is blocked until a full deck/library system exists

This phase provides a minimal solution: hardcoded starting hands that unblock UI development without building the full deck/draw infrastructure.

## What We Get When Done

When a game starts:
- Each player has 7 cards in their hand
- The cards are a mix of 2 lands and 5 creatures
- Cards use existing test card definitions from the engine
- GameSnapshot exposes these cards correctly

## Requirements

### Starting Hand Composition

Each player receives exactly 7 cards:
- 2 Basic lands
- 5 Creatures with varying power/toughness

### Card Definitions

Use existing card patterns from the engine's test helpers. The exact cards are an implementation detail, but should include:
- Lands that produce mana (or at minimum are valid land permanents)
- Creatures with defined power/toughness values
- At least some variety (not all identical cards)

### Timing

Cards appear in hand when the game transitions to "started" state. The exact mechanism is an implementation decision.

### GameSnapshot Correctness

After game creation and start:
- `snapshot.privatePlayerState.hand` contains the player's 7 cards
- `snapshot.opponentStates[0].hand.count` shows 7 (cards hidden, count visible)
- Card data includes: instanceId, name, types, power/toughness (for creatures)

## Acceptance Criteria

### Game Start
- [ ] When a new game is created and started, each player has 7 cards in hand
- [ ] Hand contains exactly 2 lands and 5 creatures

### Card Data
- [ ] Each card has a unique instanceId
- [ ] Each card has a valid name
- [ ] Each card has correct types (LAND or CREATURE)
- [ ] Creatures have power and toughness values

### GameSnapshot Integration
- [ ] GET /api/games/[gameId]/state returns snapshot with cards in hand
- [ ] Player's own hand shows full card details
- [ ] Opponent's hand shows only card count (7)

### Validation
- [ ] Creating multiple games produces consistent starting hands
- [ ] Both players receive the same card composition (2 lands, 5 creatures)

## Out of Scope

These are explicitly NOT part of this phase:
- Library zone
- Drawing cards
- Shuffling
- Mulligan
- Deck building
- Custom deck loading
- Different starting hands per player
- Random card selection

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- Phase 1b complete (basic game info displayed)
- Existing test card definitions in engine

## Testing the Feature

To verify this feature works:

1. Navigate to `/debug`
2. Create a new game
3. Start the game
4. Call GET /api/games/[gameId]/state
5. Verify `privatePlayerState.hand` contains 7 cards
6. Verify hand composition: 2 lands, 5 creatures
7. Verify opponent's hand shows count of 7

## What This Enables

With this phase complete:
- Phase 1c (Battlefield Display) can show cards after "play land" action
- Phase 1d (Hand Display) can render real cards
- UI testing has meaningful data to work with
- No artificial blockers for frontend development

## Open Questions

None. This is a well-defined bootstrap mechanism.

---

## Implementation Decisions

These decisions were made by the product manager to resolve implementation gaps.

### Decision 1: Bootstrap Location

**Location:** `StartGameCommandHandler` (application layer)

**Rationale:**
- The application layer is the correct place for orchestration logic
- `game.start()` remains a pure domain method focused on lifecycle transitions
- Adding cards to hands is a bootstrap concern, not a rules concern
- Follows existing pattern where handlers call domain methods
- Keeps the domain model pure and testable without bootstrap baggage

**Alternative rejected:** New command adds unnecessary complexity for a temporary mechanism.

### Decision 2: Card Definition Approach

**Approach:** Static card factory functions defined directly in the handler

**Rationale:**
- No CardRegistry exists in the codebase
- Creating a registry is overkill for a temporary bootstrap mechanism
- Test helpers already define card patterns inline
- Static definitions are simple, explicit, and consistent with existing patterns
- When real deck systems are implemented, this code will be deleted
- YAGNI: No infrastructure for temporary bootstrap code

### Decision 3: Specific Card Composition

Each player receives exactly these 7 cards:

**Lands (2):**
| Card Name | Types | Notes |
|-----------|-------|-------|
| Forest | LAND | Green mana source (conceptually) |
| Plains | LAND | White mana source (conceptually) |

**Creatures (5):**
| Card Name | Power/Toughness | Keywords | Notes |
|-----------|-----------------|----------|-------|
| Grizzly Bears | 2/2 | None | Vanilla baseline |
| Elite Vanguard | 2/1 | None | Aggressive stats |
| Giant Spider | 2/4 | Reach | Defensive + keyword |
| Serra Angel | 4/4 | Flying, Vigilance | Powerful + multiple keywords |
| Llanowar Elves | 1/1 | None | Small creature |

**Card composition rationale:**
- Mix of power/toughness values enables meaningful combat testing
- Includes static abilities (Flying, Reach, Vigilance) that the engine already supports
- Uses recognizable MTG card names for clarity
- Variety prevents "all cards look the same" in UI testing
- Cards are simple enough to not require complex rules interactions

### Decision 4: Phase 2 Dependency

**Action:** Phase 2 spec will be updated to explicitly list Phase 1b.5 as a dependency.

While the current "Phase 1 complete" statement is technically accurate, being explicit about Phase 1b.5 prevents confusion about the dependency chain.

---

## Ready for Implementation

This spec is now fully specified and ready for `tech-lead-strategist` to plan implementation.

**Implementation summary:**
1. Modify `StartGameCommandHandler.handle()` to populate hands after `game.start()`
2. Define 7 card factory functions within the handler (static, no registry)
3. Use existing helper patterns (`createTestLand`, `createTestCreature` style)
4. Cards added via direct hand push (acceptable for bootstrap - not gameplay mutation)
5. Update Phase 2 spec dependency list

---

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-12
**Completed**: 2026-01-12
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Add Card Factory Functions ✅
- [x] Define `createForest(ownerId)` card factory
- [x] Define `createPlains(ownerId)` card factory
- [x] Define `createGrizzlyBears(ownerId)` card factory
- [x] Define `createEliteVanguard(ownerId)` card factory
- [x] Define `createGiantSpider(ownerId)` with REACH keyword
- [x] Define `createSerraAngel(ownerId)` with FLYING + VIGILANCE
- [x] Define `createLlanowarElves(ownerId)` card factory

#### Phase 2: Implement populateStartingHands ✅
- [x] Create `populateStartingHands(game: Game)` function
- [x] Iterate through all players and create 7 cards per player
- [x] Push cards to player hand via `game.getPlayerState(playerId).hand.cards.push()`

#### Phase 3: Integrate into StartGameCommandHandler ✅
- [x] Import uuid for unique instanceIds
- [x] Add call to `populateStartingHands(game)` after `game.start()`
- [x] Verify error handling unchanged

#### Phase 4: Update Tests ✅
- [x] Add test: "populates each player's hand with 7 cards on game start"
- [x] Add test: "hand contains 2 lands and 5 creatures"
- [x] Add test: "each card has unique instanceId"
- [x] Add test: "creatures have correct power/toughness and keywords"
- [x] Update mock CardRegistry in GameSnapshot tests with new card names
- [x] Verify all existing StartGameCommand tests pass

#### Phase 5: Integration Verification ✅
- [x] Run `bun test` - all tests pass
- [x] Run `bun run lint && bun run format` - passes
- [x] Manual verification: GET /api/games/[gameId]/state returns hand with 7 cards

**Blockers**: None

**Notes**:
- Implementation completed following TDD (tests written first, watched fail, then implemented)
- Card factory functions created as static helpers in StartGameCommandHandler
- populateStartingHands() called after game.start() in handler
- All tests pass including new tests for hand composition and card properties
- MockCardRegistry updated in GameSnapshot tests to support new card names
- Lint and format checks pass
