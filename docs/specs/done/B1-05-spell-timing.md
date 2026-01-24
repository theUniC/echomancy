# Backend: Spell Timing Validation

## Overview

Implement timing restrictions for spell casting - when players can legally cast spells based on card type, game phase, and keywords. This is the rules layer that determines whether a spell can be cast, separate from the casting mechanics themselves.

**Design Goals:**
- Enforce MTG timing rules for sorceries, instants, and creatures
- Block illegal casts before they reach the stack
- Enable Flash keyword to bypass sorcery-speed restrictions
- Provide clear error messages for UI feedback

**Relationship to Other Systems:**
- Works with turn structure system (already implemented)
- Validates timing before mana payment and targeting
- Refactors existing CAST_SPELL timing validation (currently only checks main phase)
- Refactors CanCastSpell specification (needs full rewrite for instant vs sorcery speed)

**Pre-requisite:**
- Add FLASH to StaticAbility type in CardDefinition.ts (currently only has FLYING, REACH, VIGILANCE, HASTE)

## User Stories

**As a player**, I want sorcery-speed spells to be uncastable outside my main phase, so I can't accidentally make illegal plays.

**As a player**, I want instant-speed spells to be castable during opponent's turn, so I can respond to their actions.

**As a player**, I want Flash creatures to be playable as instants, so I can surprise block or hold up interaction.

## Player Experience

### Successful Casting Flow

**Scenario 1: Main Phase Sorcery**
1. Player's main phase, stack is empty
2. Player clicks Lightning Bolt (sorcery) in hand
3. Spell moves to stack successfully
4. Timing validated: main phase + empty stack + sorcery speed

**Scenario 2: Instant on Opponent's Turn**
1. Opponent's combat phase, priority passed to player
2. Player clicks Giant Growth (instant) in hand
3. Spell moves to stack successfully
4. Timing validated: instant speed works anytime with priority

**Scenario 3: Flash Creature During Combat**
1. Opponent declares attackers
2. Player clicks Snapcaster Mage (creature with Flash) in hand
3. Creature spell moves to stack successfully
4. Timing validated: Flash bypasses sorcery-speed restriction

### Rejected Casting Flow

**Scenario 4: Sorcery at Wrong Time**
1. Opponent's turn, priority passed to player
2. Player clicks Lightning Bolt (sorcery) in hand
3. Engine rejects cast
4. Error message: "Sorceries can only be cast during your main phase when the stack is empty"

**Scenario 5: Creature Without Flash**
1. Opponent's turn, priority passed to player
2. Player clicks Grizzly Bears (creature without Flash) in hand
3. Engine rejects cast
4. Error message: "Creatures can only be cast during your main phase when the stack is empty (unless they have Flash)"

## Game Rules & Mechanics

### Timing Categories (per MTG Comprehensive Rules)

**Sorcery Speed** (most restrictive):
- Active player's turn only
- Main phase only (FIRST_MAIN or SECOND_MAIN)
- Stack must be empty
- Player must have priority

**Applies to:**
- Sorceries
- Creatures (without Flash)
- Enchantments (without Flash)
- Artifacts (without Flash)
- Planeswalkers

**Instant Speed** (least restrictive):
- Any time player has priority
- Any phase, any turn
- Stack can have items on it

**Applies to:**
- Instants
- Any permanent with Flash keyword

### Flash Keyword

**Rules Text:** "You may cast this spell any time you could cast an instant."

**Behavior:**
- If a card has Flash in its staticAbilities, it uses instant-speed timing instead of sorcery-speed
- Applies to creatures, enchantments, and artifacts

### Edge Cases

**Land Cards:**
- Lands are NOT spells - they don't use this timing system
- Land playing has separate validation (one per turn, main phase, etc.)

**Abilities:**
- Activated abilities are NOT spells
- They have their own timing rules (most are instant-speed)
- Not covered by spell timing validation

**Stack Not Empty:**
- Sorcery speed requires empty stack
- Instant speed allows casting with items on stack
- This is the KEY difference between the two speeds

**Priority on Opponent's Turn:**
- Player can have priority on opponent's turn (after they pass, during combat, etc.)
- This allows instant-speed spells only
- Sorcery-speed spells are blocked

### Validation Order

Timing validation happens FIRST in the casting process:
1. Validate timing (this feature)
2. Validate mana cost can be paid
3. Validate targets are legal
4. Pay mana and add to stack

If timing fails, no mana is spent and game state is unchanged.

## Acceptance Criteria

### Sorcery Speed Validation
- [ ] Sorcery castable in main phase with empty stack on own turn
- [ ] Sorcery rejected outside main phase (even on own turn with empty stack)
- [ ] Sorcery rejected with non-empty stack (even during main phase on own turn)
- [ ] Sorcery rejected on opponent's turn (even with priority during their main phase)
- [ ] Sorcery rejected when player has priority but is not the active player
- [ ] Creature without Flash follows same rules as sorcery

### Instant Speed Validation
- [ ] Instant castable in any phase when player has priority
- [ ] Instant castable on opponent's turn
- [ ] Instant castable with non-empty stack
- [ ] Instant castable during combat phases

### Flash Keyword
- [ ] FLASH added to StaticAbility type
- [ ] Creature with Flash castable at instant speed
- [ ] Creature without Flash restricted to sorcery speed
- [ ] Flash checked via staticAbilities array
- [ ] Enchantment/Artifact with Flash also works at instant speed

### Error Messages
- [ ] Clear error when casting sorcery at wrong phase
- [ ] Clear error when casting sorcery with non-empty stack
- [ ] Clear error when casting sorcery on opponent's turn
- [ ] Error for creatures mentions Flash as alternative

### Integration
- [ ] CAST_SPELL action validates timing before anything else
- [ ] Game state unchanged if timing validation fails
- [ ] Timing errors are distinct from mana/targeting errors

## Out of Scope

**Not Included:**
- Leyline effects that grant Flash to all spells
- Split cards with different timing categories
- Adventure cards
- Cascade/triggered casts (bypass timing restrictions)
- Timing modification effects (Vedalken Orrery, etc.)

**Dependencies:**
- Turn structure (already implemented)
- Card type detection (already exists)
- StaticAbility system (already exists - needs FLASH added)
- CAST_SPELL action (already exists - needs timing refactor)
- CanCastSpell specification (already exists - needs complete rewrite)

## Success Metrics

This feature is successful when:
1. Players cannot cast sorcery-speed spells at instant speed
2. Players can cast instant-speed spells anytime they have priority
3. Flash keyword correctly upgrades timing to instant speed
4. Error messages clearly explain why a cast was rejected
5. No mana is spent on rejected casts

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-24
**Completed**: 2026-01-24
**Agent**: senior-backend-engineer

### Task Breakdown

#### Phase 1: Add FLASH Static Ability ✅
- [x] Add "FLASH" to StaticAbility type union in CardDefinition.ts
- [x] Add FLASH: "FLASH" to StaticAbilities constants object
- [x] Create test helper createCreatureWithFlash() in helpers.ts
- [x] Verify existing tests still pass

#### Phase 2: Create SpellTiming Domain Service ✅
- [x] Create src/echomancy/domainmodel/game/services/SpellTiming.ts
- [x] Implement isSorcerySpeed(card): boolean
- [x] Implement isInstantSpeed(card): boolean
- [x] Implement canCastAtCurrentTiming(game, playerId, card): boolean
- [x] Create unit tests for SpellTiming service
- [x] Test Flash upgrades permanents to instant speed

#### Phase 3: Add Timing Error Types ✅
- [x] Add SorceryTimingError base class to GameErrors.ts
- [x] Add NotYourTurnError with clear message
- [x] Add NotMainPhaseError with clear message
- [x] Add StackNotEmptyError with clear message
- [x] Ensure creature error messages mention Flash

#### Phase 4: Refactor CanCastSpell Specification ✅
- [x] Modify CanCastSpell to use SpellTimingService
- [x] Check if player has ANY castable spell (timing-aware)
- [x] Update existing CanCastSpell tests
- [x] Add tests for instant-speed spells on opponent's turn
- [x] Add tests for Flash creatures at instant speed

#### Phase 5: Refactor castSpell() in Game.ts ✅
- [x] Replace isMainPhase() check with SpellTimingService.validateCastTiming()
- [x] Ensure timing validation happens BEFORE mana payment
- [x] Throw appropriate specific errors for each timing violation
- [x] Add integration tests for timing validation
- [x] Verify game state unchanged on timing failures
- [x] Run full test suite and lint

**Blockers**: None

**Notes**:
- Phases 2 and 3 can run in parallel (no shared dependencies)
- All other phases are sequential
- Use senior-backend-engineer for all implementation (not subagent-driven-development)
