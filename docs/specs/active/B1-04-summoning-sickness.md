# Backend: Summoning Sickness

## Goal

Implement summoning sickness rules to prevent creatures from attacking or activating tap abilities on the turn they enter the battlefield, unless they have haste.

## What We Get When Done

Creatures cannot attack or use tap abilities the turn they enter the battlefield. This rule is checked automatically when players attempt these actions, and violations are rejected with clear error messages.

## Game Rules & Mechanics

### Summoning Sickness Rule (MTG CR 302.6)

A creature has summoning sickness if:
- It entered the battlefield this turn under the current player's control, OR
- It came under the current player's control this turn

A creature with summoning sickness cannot:
- Be declared as an attacker
- Pay costs that include the tap symbol (T) or untap symbol (Q)

### MVP Simplification

For MVP, summoning sickness applies only to creatures that entered the battlefield this turn. Control change tracking is deferred since no control-change effects exist yet.

**Implementation**: Add a boolean flag `hasSummoningSickness` to creature state.

**Flag Behavior**:
- Set to `true` when creature enters the battlefield
- Set to `false` at the start of the creature's controller's next turn
- Checked when attempting to declare attackers or activate tap abilities

### Haste Keyword

Creatures with the "HASTE" keyword ignore summoning sickness entirely. They can attack and use tap abilities the turn they enter.

**Implementation**: Check `card.definition.keywords.includes("HASTE")` before enforcing summoning sickness. This follows the same consultative pattern as Flying, Reach, and Vigilance.

## Player Experience

**From Backend Perspective**:

1. Player casts creature spell
2. Creature resolves and enters battlefield
3. Engine sets `hasSummoningSickness = true`
4. Player attempts to declare creature as attacker
5. Engine checks `hasSummoningSickness`
6. If `true`, engine rejects action with `CreatureHasSummoningSicknessError`
7. Frontend displays error to player

**Next Turn**:
1. At start of creature's controller's turn (UNTAP step)
2. Engine sets `hasSummoningSickness = false` for all creatures they control
3. Player can now attack with creature

## Acceptance Criteria

- Creatures entering battlefield have `hasSummoningSickness = true`
- Declaring attacker with summoning sickness throws `CreatureHasSummoningSicknessError`
- Activating tap ability with summoning sickness throws `CreatureHasSummoningSicknessError`
- Summoning sickness cleared at UNTAP step for active player's creatures
- Flag exported in `CreatureStateExport` for UI visibility
- **Creatures with HASTE keyword can attack the turn they enter**
- **Creatures with HASTE keyword can use tap abilities the turn they enter**
- Test coverage for creature entering battlefield, attacking same turn (fails), attacking next turn (succeeds)
- Test coverage for tap abilities with summoning sickness
- Test coverage for summoning sickness cleared on next turn
- **Test coverage for Haste bypassing summoning sickness**

## Implementation Tasks

1. Add `hasSummoningSickness: boolean` to `CreatureStateExport` type
2. Add `hasSummoningSickness: boolean` to internal creature state in battlefield cards
3. Set `hasSummoningSickness = true` when creature enters battlefield (in `enterBattlefield()`)
4. Add `CreatureHasSummoningSicknessError` to `GameErrors.ts`
5. Check summoning sickness in `declareAttacker()` action handler
6. Check summoning sickness in ability activation for tap costs
7. Clear summoning sickness for active player's creatures at UNTAP step
8. Test creature enters and cannot attack same turn
9. Test creature can attack next turn
10. Test tap ability fails with summoning sickness
11. Test summoning sickness cleared correctly

## Dependencies

- Creature state system (already exists)
- Battlefield zone (already exists)
- Turn structure and UNTAP step (already exists)
- Declare attacker action (already exists)
- Activated ability system (already exists for tap cost checking)

## Out of Scope

- **Control change tracking** - Summoning sickness when creature changes controller mid-turn (no control-change effects exist)
- **Untap symbol (Q) costs** - Only tap symbol (T) costs checked for now
- **Creatures becoming non-creatures** - Animation effects that remove/add creature type
- **Timestamp tracking** - Use simple boolean flag, not turn entry timestamp
- **"Gains haste until end of turn"** - Temporary keyword effects (no duration tracking yet)

## Notes for Implementation

- Use existing `enterBattlefield()` method to set the flag
- Clear flag in existing UNTAP step handler (where creatures untap)
- Add check before tapping creature in `declareAttacker()`
- Add check in activated ability cost payment for `TapSelfCost`
- Export flag in `Game.exportState()` for UI to display (e.g., grayed-out creatures)
- Follow existing error patterns from `GameErrors.ts`
- Keep implementation simple - just a boolean flag, no complex timestamp logic

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-17
**Completed**: 2026-01-17
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Summoning Sickness Implementation ✅

**Types & Errors**
- [x] Add `HASTE` to `StaticAbility` type in `CardDefinition.ts`
- [x] Add `HASTE` to `StaticAbilities` constant object
- [x] Add `CreatureHasSummoningSicknessError` in `GameErrors.ts`
- [x] Add `hasSummoningSickness: boolean` to `CreatureState` type in `Game.ts`
- [x] Add `hasSummoningSickness: boolean` to `CreatureStateExport` type in `GameStateExport.ts`

**Core Logic**
- [x] Set `hasSummoningSickness = true` in `initializeCreatureStateIfNeeded()`
- [x] Export `hasSummoningSickness` in `exportCreatureState()`
- [x] Check summoning sickness in `declareAttacker()` with Haste bypass
- [x] Check summoning sickness in `payActivationCost()` with Haste bypass
- [x] Clear summoning sickness at UNTAP step for active player's creatures

**Tests**
- [x] Add `createTestCreatureWithHaste()` helper
- [x] Test creature enters battlefield with `hasSummoningSickness = true`
- [x] Test creature cannot attack same turn (throws error)
- [x] Test creature can attack next turn (summoning sickness cleared)
- [x] Test tap ability fails with summoning sickness (throws error)
- [x] Test Haste creature can attack same turn
- [x] Test Haste creature can use tap abilities same turn

**Blockers**: None
**Notes**:
- All tests passing (9 new tests in Game.summoningSickness.test.ts)
- All existing tests pass (441 tests total)
- Created `addCreatureToBattlefieldWithSummoningSickness()` helper for tests that need to test summoning sickness behavior
- Updated `addCreatureToBattlefield()` helper to clear summoning sickness by default for backwards compatibility with existing tests
