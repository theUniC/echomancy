# Backend: Spell Timing Validation

## Overview

Implement timing restrictions for spell casting - when players can legally cast spells based on card type, game phase, and keywords. This is the rules layer that determines whether a spell can be cast, separate from the casting mechanics themselves.

**Design Goals:**
- Enforce MTG timing rules for sorceries, instants, and creatures
- Block illegal casts before they reach the stack
- Enable Flash keyword to bypass sorcery-speed restrictions
- Provide clear error messages for UI feedback

**Relationship to Other Systems:**
- Builds on existing spell casting system (07b-backend-spell-casting-system.md)
- Works with turn structure system (already implemented)
- Validates timing before mana payment and targeting

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
4. ✅ Timing validated: main phase + empty stack + sorcery speed

**Scenario 2: Instant on Opponent's Turn**
1. Opponent's combat phase, priority passed to player
2. Player clicks Giant Growth (instant) in hand
3. Spell moves to stack successfully
4. ✅ Timing validated: instant speed works anytime with priority

**Scenario 3: Flash Creature During Combat**
1. Opponent declares attackers
2. Player clicks Snapcaster Mage (creature with Flash) in hand
3. Creature spell moves to stack successfully
4. ✅ Timing validated: Flash bypasses sorcery-speed restriction

### Rejected Casting Flow

**Scenario 4: Sorcery at Wrong Time**
1. Opponent's turn, priority passed to player
2. Player clicks Lightning Bolt (sorcery) in hand
3. Engine rejects cast
4. Error message: "Sorceries can only be cast during your main phase when the stack is empty"
5. ❌ Timing validation failed: not player's turn

**Scenario 5: Creature Without Flash**
1. Opponent's turn, priority passed to player
2. Player clicks Grizzly Bears (creature without Flash) in hand
3. Engine rejects cast
4. Error message: "Creatures can only be cast during your main phase when the stack is empty (unless they have Flash)"
5. ❌ Timing validation failed: no Flash keyword

## Game Rules & Mechanics

### Timing Categories

**Sorcery Speed** (most restrictive):
- Active player's turn
- Main phase (FIRST_MAIN or SECOND_MAIN)
- Stack must be empty
- Player has priority

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

**Implementation:**
- Check card for `keywords: ['flash']` (case-insensitive)
- If Flash present, bypass sorcery-speed checks
- Use instant-speed timing instead

**Affected Card Types:**
- Creatures with Flash (e.g., Snapcaster Mage, Ambush Viper)
- Enchantments with Flash (e.g., Boon of Safety)
- Artifacts with Flash (e.g., Cloak of Invisibility)

### Edge Cases

**Case 1: Land Cards**
- Lands are NOT spells
- Never use this timing system
- Separate validation in `playLand()` action

**Case 2: Abilities**
- Activated abilities are NOT spells
- Handled by ability system
- Not covered by spell timing

**Case 3: Stack Not Empty**
- Sorcery speed requires empty stack
- Instant speed allows full stack
- This is KEY difference between the two

**Case 4: Priority Without Active Turn**
- Player can have priority on opponent's turn
- This allows instant-speed spells
- Blocks sorcery-speed spells

## Acceptance Criteria

### Sorcery Speed Validation
- [ ] Sorcery card castable in main phase with empty stack
- [ ] Sorcery card rejected outside main phase
- [ ] Sorcery card rejected with non-empty stack
- [ ] Sorcery card rejected on opponent's turn
- [ ] Creature card follows same rules as sorcery (unless Flash)

### Instant Speed Validation
- [ ] Instant card castable in any phase with priority
- [ ] Instant card castable on opponent's turn
- [ ] Instant card castable with non-empty stack
- [ ] Instant card castable during combat

### Flash Keyword
- [ ] Creature with Flash castable as instant
- [ ] Creature without Flash follows sorcery timing
- [ ] Flash detection is case-insensitive (`keywords.includes('flash')`)
- [ ] Error messages mention Flash when relevant

### Error Messages
- [ ] Clear error for sorcery at wrong phase
- [ ] Clear error for sorcery with stack not empty
- [ ] Error mentions Flash as alternative for creatures
- [ ] Error specifies which condition failed

### Integration
- [ ] `castSpell()` action calls timing validation first
- [ ] Validation happens before mana payment
- [ ] Validation happens before targeting
- [ ] Game state unchanged if timing invalid

## Out of Scope

**Not Included in This Spec:**
- **Leyline effects** - "You may cast sorceries as though they had Flash" (modifies timing rules globally)
- **Cost modification timing** - Convoke, delve, etc. don't affect timing
- **Split cards** - Cards with multiple timing categories (e.g., Fire/Ice)
- **Adventure cards** - Separate timing for creature vs adventure half
- **X costs** - Cost calculation is separate from timing
- **Counter spells** - Responding to spells is about priority, not timing
- **Cascade/triggered casts** - Spells cast by effects bypass timing restrictions

**Dependencies:**
- Existing spell casting system (07b)
- Turn structure (already implemented)
- Card type detection (already exists)
- Keyword system (already exists)

**Future Considerations:**
- Timing modification effects (Vedalken Orrery, Leyline of Anticipation)
- Split cards with different timings
- Adventure cards
- Bestow and mutate timing rules

## Implementation Tasks

1. **Create timing validation function** - `canCastSpellNow(game, playerId, card): boolean`
2. **Detect sorcery vs instant speed** - Based on card type and keywords
3. **Check main phase + empty stack** - For sorcery speed
4. **Check active turn** - For sorcery speed
5. **Flash keyword detection** - Bypass sorcery checks if present
6. **Error message generation** - Explain why timing failed
7. **Integrate with castSpell()** - Validate before mana/targeting
8. **Test sorcery timing** - All restriction scenarios
9. **Test instant timing** - All permissive scenarios
10. **Test Flash keyword** - Creature with/without Flash

## Notes for Implementation

### Timing Check Location

Add timing validation as FIRST step in `castSpell()`:

```
castSpell(playerId, cardId, targets):
  1. Get card from hand
  2. ❗ Validate timing (NEW)
  3. Validate mana cost
  4. Validate targets
  5. Pay mana
  6. Add to stack
```

### Sorcery Speed Logic

Sorcery speed requires ALL of:
- Active player has priority
- Current phase is FIRST_MAIN or SECOND_MAIN
- Stack is empty
- It's the active player's turn

### Instant Speed Logic

Instant speed requires ONLY:
- Player has priority

### Card Type Detection

Use existing card type field:
- `card.type === 'sorcery'` → sorcery speed
- `card.type === 'instant'` → instant speed
- `card.type === 'creature'` → sorcery speed (unless Flash)
- `card.type === 'enchantment'` → sorcery speed (unless Flash)
- `card.type === 'artifact'` → sorcery speed (unless Flash)
- `card.type === 'planeswalker'` → sorcery speed (Flash doesn't apply)

### Flash Detection

Check keywords array:
```typescript
const hasFlash = card.keywords?.some(
  kw => kw.toLowerCase() === 'flash'
) ?? false
```

### Error Message Template

**Sorcery speed failed:**
"Cannot cast [card name]: Sorceries can only be cast during your main phase when the stack is empty."

**Creature without Flash:**
"Cannot cast [card name]: Creatures can only be cast during your main phase when the stack is empty (unless they have Flash)."

**Wrong phase:**
"Cannot cast [card name]: Sorcery-speed spells require a main phase."

**Stack not empty:**
"Cannot cast [card name]: Sorcery-speed spells require an empty stack."

### Testing Strategy

**Unit Tests:**
- Timing validation function in isolation
- All timing scenarios with mock game state

**Integration Tests:**
- Full `castSpell()` flow with timing validation
- Rejection before mana payment
- Successful cast after validation passes

**Test Coverage:**
- Sorcery in main phase (pass)
- Sorcery outside main phase (fail)
- Sorcery with stack not empty (fail)
- Instant in main phase (pass)
- Instant outside main phase (pass)
- Instant with stack not empty (pass)
- Creature with Flash (pass as instant)
- Creature without Flash (fail outside main)
- Flash keyword case-insensitive (pass)

---

**End of Specification**
