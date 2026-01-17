# Backend: Spell Casting System

## Goal

Enable the engine to cast spells from hand with mana payment and targeting, putting them on the stack for resolution.

## What We Get When Done

Players can cast instant/sorcery spells, pay their mana costs automatically, select targets if required, and resolve effects correctly.

## Game Rules & Mechanics

### Spell Casting Flow

1. **Announce**: Move card from hand to stack
2. **Choose Targets**: If spell requires targets, select valid ones
3. **Pay Costs**: Deduct mana from player's pool (auto-pay for MVP)
4. **Resolution**: When resolving, apply effects and move to graveyard

### Mana Cost Format

Mana costs stored as `{ generic: number, W?: number, U?: number, B?: number, R?: number, G?: number, C?: number }`

**Examples**:
- "2UU" → `{ generic: 2, U: 2 }`
- "1WW" → `{ generic: 1, W: 2 }`
- "4" → `{ generic: 4 }`
- "BBB" → `{ B: 3 }`

### Auto-Pay Logic (MVP)

For MVP, use simple auto-pay:
1. Pay colored requirements first (U, U match against U, U in pool)
2. Pay generic cost with any remaining mana (prefer colorless C, then any color)
3. Fail if insufficient mana

**Example**: Cost `{ generic: 2, U: 2 }` with pool `{ U: 2, G: 1, C: 1 }`
- Pay U, U from pool → pool becomes `{ G: 1, C: 1 }`
- Pay generic: 2 from C (1) + G (1) → pool becomes empty
- Success

### Targeting Rules

- Spell declares target requirements (e.g., "target creature")
- Valid targets filtered by engine
- Frontend receives valid target list
- Frontend submits selected target(s)
- Engine validates and locks targets on stack

### Resolution

When spell resolves from stack:
1. Check targets still valid (not removed from battlefield)
2. Apply effect to targets
3. Move spell card to graveyard
4. Continue priority/stack flow

## Player Experience

**From Backend Perspective** (what the frontend calls):

1. Frontend calls `castSpell(playerId, cardId, targets?)`
2. Engine validates timing, mana, targets
3. Engine applies mana payment
4. Engine adds spell to stack
5. Engine returns updated game state
6. Frontend renders new state (spell on stack, mana depleted)

When spell resolves:
1. Engine pops from stack
2. Applies effects
3. Moves to graveyard
4. Frontend receives state update

## Acceptance Criteria

- [ ] Engine accepts `castSpell` action with card ID and optional targets
- [ ] Mana cost parsing from string format (e.g., "2UU")
- [ ] Auto-pay deducts correct mana from pool
- [ ] Spell placed on stack with locked targets
- [ ] Resolution applies effects and moves card to graveyard
- [ ] Invalid casts rejected (wrong timing, insufficient mana, invalid targets)
- [ ] Test coverage for multi-color costs, generic costs, colorless costs
- [ ] Test coverage for targeted and non-targeted spells

## Implementation Tasks

1. Add `ManaCost` type and parsing logic for mana cost strings
2. Implement auto-pay algorithm in mana pool
3. Add `castSpell()` game action handler
4. Validate casting legality (timing, mana, targets)
5. Add spell to stack with targets
6. Test mana payment edge cases
7. Test spell resolution flow

## Dependencies

- Mana pool system (already exists)
- Stack system (already exists)
- Zone transitions (already exists)
- Priority system (already exists)

## Out of Scope

- **Manual mana payment** - User chooses which lands to tap (future UI feature)
- **Mana ability activation** - Tap lands manually to generate mana (use auto-tap for now)
- **X costs** - `{ X: true, generic: 0 }` requires separate spec
- **Alternative costs** - Flashback, overload, kicker, etc.
- **Split costs** - Hybrid mana `(W/U)` or Phyrexian mana
- **Countering spells** - Counter spell mechanics (separate feature)
- **Copy effects** - "Copy target spell" (future)
- **Mana restrictions** - "Mana from this source can only be used to cast creature spells"

## Notes for Implementation

- Use existing `game.apply()` mutation pattern
- Follow DDD patterns in `src/echomancy/engine/domain/`
- Mana cost parsing should be a pure utility function
- Auto-pay should be a method on `ManaPool` class
- Spell casting should integrate with existing stack/priority logic
- Keep MVP simple - no optimization, just first-fit mana payment
