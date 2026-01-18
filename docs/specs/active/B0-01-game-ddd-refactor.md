# B0-01: Game.ts DDD Refactor

## Goal

Reduce Game.ts from 2,282 lines to ~600-800 lines by extracting domain concepts into proper DDD building blocks: Value Objects, Entities, Domain Services, and Specifications.

## Problem

Game.ts is a monolithic class mixing:
- Creature state management (P/T, counters, damage)
- Mana pool operations
- Combat resolution
- Stack management
- Turn structure
- Trigger evaluation
- State-based actions
- Validation predicates

This makes the code hard to navigate, test, and maintain.

## What We Get When Done

```
domainmodel/game/
├── Game.ts                  (~600-800 lines - Aggregate Root)
│
├── valueobjects/
│   ├── ManaPool.ts          (immutable, with add/spend/clear)
│   ├── CreatureState.ts     (P/T, damage, counters, summoningSickness)
│   ├── TurnState.ts         (step, phase, activePlayer, turnNumber)
│   └── CombatState.ts       (attackers, blockers mapping)
│
├── entities/
│   ├── Battlefield.ts       (zone with permanents, tap/untap)
│   ├── Hand.ts              (private zone)
│   ├── Graveyard.ts         (public zone)
│   └── TheStack.ts          (LIFO resolution stack)
│
├── services/
│   ├── CombatResolution.ts  (calculate and apply combat damage)
│   ├── TriggerEvaluation.ts (find and execute triggers)
│   └── StateBasedActions.ts (lethal damage, 0 toughness)
│
└── specifications/
    ├── CanPlayLand.ts
    ├── CanCastSpell.ts
    ├── CanDeclareAttacker.ts
    ├── CanDeclareBlocker.ts
    ├── CanActivateAbility.ts
    └── HasPriority.ts
```

Game.ts remains the Aggregate Root that:
- Owns all state
- Coordinates operations
- Enforces invariants
- Exposes public API (apply, exportState, getAllowedActionsFor)

---

## DDD Building Blocks

### Value Objects
- Immutable
- Equality by value, not reference
- No identity
- Examples: ManaPool, CreatureState

### Entities
- Have identity (instanceId)
- Mutable state
- Lifecycle
- Examples: Battlefield, TheStack

### Domain Services
- Stateless
- Operations that don't belong to a single entity
- Pure functions that operate on aggregates
- Examples: CombatResolution, TriggerEvaluation

### Specifications
- Encapsulate business rules as objects
- `isSatisfiedBy(game, context): boolean`
- Composable (and, or, not)
- Examples: CanPlayLand, HasPriority

---

## Phases

### Phase 1: Value Objects (Low Risk)

**Extract:**
- `ManaPool` - lines 837-978
- `CreatureState` - lines 699-835

**Acceptance Criteria:**
- [ ] ManaPool is immutable class with add(), spend(), clear() returning new instances
- [ ] CreatureState holds P/T, damage, counters, hasSummoningSickness
- [ ] Game.ts uses these VOs instead of inline Maps
- [ ] All 441+ tests pass
- [ ] Game.ts reduced by ~250 lines

**Tasks:**
1. Create `valueobjects/ManaPool.ts` with immutable operations
2. Create `valueobjects/CreatureState.ts` with P/T calculation
3. Refactor Game.ts to use ManaPool VO
4. Refactor Game.ts to use CreatureState VO
5. Run tests, lint, format

---

### Phase 2: Specifications (Low Risk)

**Extract:**
- All `can*` and `has*` predicates - lines 2080-2212

**Acceptance Criteria:**
- [ ] Each specification is a class with `isSatisfiedBy(game): boolean`
- [ ] Specifications are used in Game.ts and getAllowedActionsFor()
- [ ] All tests pass
- [ ] Game.ts reduced by ~130 lines

**Tasks:**
1. Create `specifications/Specification.ts` interface
2. Create `specifications/CanPlayLand.ts`
3. Create `specifications/CanCastSpell.ts`
4. Create `specifications/CanDeclareAttacker.ts`
5. Create remaining specs (HasPriority, CanActivateAbility, etc.)
6. Refactor Game.ts to use specifications
7. Run tests, lint, format

---

### Phase 3: Zone Entities (Medium Risk)

**Extract:**
- Battlefield logic
- Hand logic
- Graveyard logic

**Acceptance Criteria:**
- [ ] Battlefield entity manages permanents, tap/untap, creature states
- [ ] Hand entity manages private cards
- [ ] Graveyard entity manages public cards
- [ ] Zone transitions handled by entities
- [ ] All tests pass
- [ ] Game.ts reduced by ~150 lines

**Tasks:**
1. Create `entities/Battlefield.ts` with permanent management
2. Create `entities/Hand.ts`
3. Create `entities/Graveyard.ts`
4. Refactor PlayerState to use zone entities
5. Run tests, lint, format

---

### Phase 4: TheStack Entity (Medium Risk)

**Extract:**
- Stack management - lines 1186-1235, 1588-1655

**Acceptance Criteria:**
- [ ] TheStack entity manages spells and abilities
- [ ] Push, pop, peek, resolve operations
- [ ] All tests pass
- [ ] Game.ts reduced by ~100 lines

**Tasks:**
1. Create `entities/TheStack.ts`
2. Move stack item types to TheStack
3. Refactor Game.ts to use TheStack
4. Run tests, lint, format

---

### Phase 5: Domain Services (Higher Risk)

**Extract:**
- CombatResolution - lines 1773-1931
- TriggerEvaluation - lines 1970-2074
- StateBasedActions - lines 1933-1968

**Acceptance Criteria:**
- [ ] CombatResolution.resolve(game) calculates and applies damage
- [ ] TriggerEvaluation.evaluate(game, event) finds and executes triggers
- [ ] StateBasedActions.perform(game) handles lethal damage, etc.
- [ ] Services are stateless (receive Game, operate on it)
- [ ] All tests pass
- [ ] Game.ts reduced by ~250 lines

**Tasks:**
1. Create `services/CombatResolution.ts`
2. Create `services/TriggerEvaluation.ts`
3. Create `services/StateBasedActions.ts`
4. Refactor Game.ts to call services
5. Run tests, lint, format

---

### Phase 6: TurnState & CombatState VOs (Medium Risk)

**Extract:**
- Turn structure state
- Combat state (attackers/blockers)

**Acceptance Criteria:**
- [ ] TurnState VO holds step, phase, activePlayer, turnNumber
- [ ] CombatState VO holds attacker/blocker assignments
- [ ] All tests pass
- [ ] Game.ts cleaner with grouped state

**Tasks:**
1. Create `valueobjects/TurnState.ts`
2. Create `valueobjects/CombatState.ts`
3. Refactor Game.ts to use these VOs
4. Run tests, lint, format

---

### Phase 7: Final Cleanup

**Tasks:**
1. Review Game.ts for remaining extraction opportunities
2. Ensure consistent patterns across all extracted components
3. Update documentation in `docs/architecture.md`
4. Final test run and code review

**Acceptance Criteria:**
- [ ] Game.ts is 600-800 lines
- [ ] All extracted components have unit tests
- [ ] Architecture documentation updated
- [ ] All 441+ tests pass

---

## Dependencies

- None (this is B0 priority - foundational)

## Blocks

- All other backlog items should wait until at least Phase 1-2 complete

## Out of Scope

- Changing Game.ts public API (must remain backward compatible)
- Adding new features
- Event sourcing
- Performance optimization
- UI changes

## Risk Mitigation

- Each phase is a separate commit
- Tests must pass before moving to next phase
- Can rollback any phase independently
- Start with lowest-risk extractions (VOs, Specs)

## Estimated Effort

| Phase | Effort | Risk |
|-------|--------|------|
| 1: Value Objects | 3-4h | Low |
| 2: Specifications | 3-4h | Low |
| 3: Zone Entities | 4-5h | Medium |
| 4: TheStack | 2-3h | Medium |
| 5: Domain Services | 5-6h | Higher |
| 6: TurnState/CombatState | 2-3h | Medium |
| 7: Final Cleanup | 2-3h | Low |

**Total: ~22-28 hours**

## Success Criteria

- Game.ts reduced from 2,282 to ~600-800 lines
- All components independently testable
- Clear domain language in code (Battlefield, ManaPool, TheStack)
- No "Manager", "Handler", "System" names
- All existing tests pass

---

## Implementation Tracking

**Status**: In Progress
**Started**: 2026-01-18
**Completed**:
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Value Objects (Low Risk) ✅
- [x] Create `valueobjects/ManaPool.ts` with immutable operations
- [x] Create `valueobjects/CreatureState.ts` with P/T calculation
- [x] Refactor Game.ts to use ManaPool VO
- [x] Refactor Game.ts to use CreatureState VO
- [x] Run tests, lint, format

#### Phase 2: Specifications (Low Risk) ✅
- [x] Create `specifications/Specification.ts` interface
- [x] Create `specifications/CanPlayLand.ts`
- [x] Create `specifications/CanCastSpell.ts`
- [x] Create `specifications/CanDeclareAttacker.ts`
- [x] Create remaining specs (HasPriority, CanActivateAbility, etc.)
- [x] Refactor Game.ts to use specifications
- [x] Run tests, lint, format

#### Phase 3: Zone Entities (Medium Risk) ✅
- [x] Create `entities/Battlefield.ts` with permanent management
- [x] Create `entities/Hand.ts`
- [x] Create `entities/Graveyard.ts`
- [x] Refactor PlayerState to use zone entities
- [x] Run tests, lint, format

##### Phase 3 Detailed Breakdown:

###### Phase 3.1: Battlefield Entity (Medium) ✅
- [x] Create `entities/Battlefield.ts` with addPermanent, removePermanent, findPermanent methods
- [x] Create unit tests for Battlefield entity
- [x] Update PlayerState type to use Battlefield entity
- [x] Refactor Game.ts enterBattlefield() and movePermanentToGraveyard() to use entity

###### Phase 3.2: Hand Entity (Small) ✅
- [x] Create `entities/Hand.ts` with addCard, removeCard, findCard methods
- [x] Create unit tests for Hand entity
- [x] Update PlayerState to use Hand entity
- [x] Refactor Game.ts playLand() and castSpell() to use entity

###### Phase 3.3: Graveyard Entity (Small) ✅
- [x] Create `entities/Graveyard.ts` with addCard, getAllCards, count methods
- [x] Create unit tests for Graveyard entity
- [x] Update PlayerState to use Graveyard entity
- [x] Refactor Game.ts getGraveyard() to use entity

###### Phase 3.4: Integration & Verification (Medium) ✅
- [x] Update exportZone() helper in Game.ts (works with entities via duck typing)
- [x] Update test helpers to work with new zone entities (entities expose mutable `cards` getter)
- [x] Run full test suite, fix any issues (561 pass, 0 fail)
- [x] Run lint and format
- [x] Zone entities integrated - backward compatible via mutable `cards` getter

#### Phase 4: TheStack Entity (Medium Risk) ✅
- [x] Create `entities/TheStack.ts` with push, pop, peek, isEmpty, hasItems methods
- [x] Create unit tests for TheStack entity (22 tests)
- [x] Stack types remain in StackTypes.ts (no need to move - already well-organized)
- [x] Refactor Game.ts to use TheStack (backward compatible via mutable `items` getter)
- [x] Run tests, lint, format (583 pass, 0 fail)

#### Phase 5: Domain Services (Higher Risk) 🔄
- [ ] Create `services/CombatResolution.ts`
- [ ] Create `services/TriggerEvaluation.ts`
- [x] Create `services/StateBasedActions.ts` with `findCreaturesToDestroy(game)` function
- [x] Add `getCreatureEntries()` public method to Game.ts for service access
- [x] Refactor `performStateBasedActions()` to use StateBasedActions service
- [x] Run tests, lint, format (591 pass, 0 fail)

**Note**: CombatResolution and TriggerEvaluation require more Game internals exposed.
Pattern established with StateBasedActions - remaining services can follow same approach.

#### Phase 6: TurnState & CombatState VOs (Medium Risk) ⏳
- [ ] Create `valueobjects/TurnState.ts`
- [ ] Create `valueobjects/CombatState.ts`
- [ ] Refactor Game.ts to use these VOs
- [ ] Run tests, lint, format

#### Phase 7: Final Cleanup ⏳
- [ ] Review Game.ts for remaining extraction opportunities
- [ ] Ensure consistent patterns across all extracted components
- [ ] Update documentation in `docs/architecture.md`
- [ ] Final test run and code review

**Blockers**: None

**Notes**:
- Phase 1 & 2 completed prior to this planning session
- Phase 3 focuses on zone entities: Battlefield, Hand, Graveyard
- Zone entities are entities (not value objects) because they have identity and mutable state
- All zone transitions remain in Game.ts; entities just manage their card arrays
