# Combat Resolution System

This document describes the combat resolution system in Echomancy, including damage assignment, creature destruction, and damage to players.

## Overview

The combat resolution system implements the basic Magic: The Gathering combat flow:

1. Declare attackers (DECLARE_ATTACKERS step)
2. Declare blockers (DECLARE_BLOCKERS step)
3. Assign and resolve damage (COMBAT_DAMAGE step)
4. State-based actions destroy creatures with lethal damage
5. Combat state resets (END_OF_COMBAT step)
6. Damage is cleared (CLEANUP step)

## Combat Flow

### 1. Declare Attackers

During the DECLARE_ATTACKERS step, the active player can declare creatures as attackers.

Restrictions:
- Only untapped creatures can attack
- A creature can only attack once per turn
- The creature becomes tapped when declared as an attacker

### 2. Declare Blockers

During the DECLARE_BLOCKERS step, the defending player can assign blockers to attackers.

Restrictions:
- Only untapped creatures can block
- A creature can only block one attacker per combat
- **MVP Limitation:** Each attacker can only be blocked by one creature (1-to-1 blocking only)

### 3. Combat Damage

During the COMBAT_DAMAGE step, damage is automatically assigned and resolved:

**For blocked attackers:**
- The attacker deals damage equal to its power to each blocker
- Each blocker deals damage equal to its power back to the attacker
- Damage is simultaneous (both creatures deal damage before either dies)

**For unblocked attackers:**
- The attacker deals damage equal to its power to the defending player
- Player life total is reduced by the damage amount

### 4. State-Based Actions

Immediately after damage is assigned, state-based actions check for creatures with lethal damage:

- If a creature has damage marked ≥ its toughness, it is destroyed
- Destroyed creatures are moved to their owner's graveyard
- This happens before any player receives priority

## Damage Tracking

### Damage Marked This Turn

Each creature tracks damage marked on it during the current turn in `damageMarkedThisTurn`.

- Damage accumulates (multiple sources add together)
- Damage is compared to current toughness to determine lethality
- Damage is cleared at the CLEANUP step

### Damage Cleanup

All damage marked on creatures is cleared at the CLEANUP step:

- `damageMarkedThisTurn` is reset to 0 for all creatures
- This happens once per turn, not per phase
- Damage does not persist across turns

## Combat State

### Blocking Relationships

Each creature tracks its combat involvement:

- `isAttacking`: true if currently declared as an attacker
- `blockingCreatureId`: the creature this creature is blocking (null if not blocking)
- `blockedBy`: array of creatures blocking this attacker

### State Cleanup

Combat state is cleared at the END_OF_COMBAT step:

- `isAttacking` is set to false
- `blockingCreatureId` is set to null
- `blockedBy` is cleared

This allows creatures to potentially attack again if there is an extra combat phase.

### Turn-Based Reset

Additional state resets when the turn changes:

- `hasAttackedThisTurn` is cleared
- All combat state is cleared
- Damage is cleared (happens at CLEANUP before turn change)

## Player Damage

Players track their life total directly:

- Starting life total is 20 (default)
- Unblocked attackers reduce player life by their power
- Multiple attackers deal cumulative damage
- No loss condition is checked (MVP behavior)

## Game Methods

### Combat Actions

**declareAttacker(playerId, creatureId)**
- Validates the creature can attack
- Sets attacking state and taps the creature
- Emits CREATURE_DECLARED_ATTACKER event

**declareBlocker(playerId, blockerId, attackerId)**
- Validates the blocker can block
- Establishes blocking relationship
- No event emitted (deferred for future expansion)

### Damage Methods

**resolveCombatDamage()**
- Called automatically at COMBAT_DAMAGE step
- Assigns damage to all creatures in combat
- Deals damage to defending player from unblocked attackers

**performStateBasedActions()**
- Called after damage resolution
- Destroys creatures with lethal damage
- Future: will handle 0-toughness, player loss, etc.

**clearDamageOnAllCreatures()**
- Called at CLEANUP step
- Resets damageMarkedThisTurn to 0 for all creatures

## MVP Limitations

The following combat features are intentionally excluded from the MVP:

**Damage Assignment:**
- First strike / Double strike
- Trample
- Deathtouch
- Damage assignment order for multiple blockers

**Damage Modification:**
- Damage prevention
- Damage redirection to planeswalkers
- Damage replacement effects

**Combat Abilities:**
- Indestructible
- Regeneration
- Static abilities modifying power/toughness during combat
- Combat damage triggers (implemented but limited)

**Other Limitations:**
- Player loss condition not checked
- 0-toughness state-based action not implemented
- No instant-speed interaction during combat

These limitations are documented with TODO comments in the source code and will be addressed in future expansions.

## Testing

Comprehensive tests validate the combat resolution system:

- Creature vs creature combat with various power/toughness combinations
- Multiple creatures attacking and blocking
- Damage to players from unblocked attackers
- Simultaneous damage resolution
- Damage timing and cleanup
- Counter interactions with combat
- Edge cases (tapped creatures, double blocking, etc.)

See `Game.combatResolution.test.ts` for the full test suite.

## Integration with Other Systems

**Creature Stats:**
- Uses getCurrentPower() and getCurrentToughness() for damage calculations
- Respects +1/+1 counters and other stat modifiers

**Trigger System:**
- CREATURE_DECLARED_ATTACKER event fires when creature attacks
- Future: CREATURE_DECLARED_BLOCKER event when needed
- State-based actions trigger ZONE_CHANGED events for destroyed creatures

**Turn Structure:**
- Integrates with step-based turn progression
- Automatic combat damage at COMBAT_DAMAGE step
- Automatic cleanup at CLEANUP step

## Design Principles

The combat resolution system follows Echomancy's core principles:

1. **Correctness**: Implements actual Magic rules faithfully
2. **Explicitness**: All damage and state changes are explicit
3. **Testability**: Pure functions with no hidden state
4. **Simplicity**: MVP excludes complex features without compromising correctness
5. **Documented limitations**: All exclusions are clearly marked with TODOs
