# Ability Contract – MVP Definition

This document summarizes the formal Ability contract for Echomancy.

## Overview

This update formalizes the contract for Abilities in Echomancy without introducing new systems. It clarifies:
- What an Ability is
- When abilities are evaluated
- How abilities interact with the stack
- What context abilities receive
- What is explicitly OUT of scope for the MVP

## What Changed

### New Files

**`src/echomancy/domainmodel/abilities/Ability.ts`**
- Complete formal contract definition for abilities
- Defines supported types (ActivatedAbility, TriggeredAbility)
- Documents explicitly unsupported types (StaticAbility, ReplacementEffect, etc.)
- Clarifies stack interaction rules
- Documents when abilities are evaluated
- Lists all MVP limitations and non-goals

### Updated Files

**`src/echomancy/domainmodel/abilities/ActivatedAbility.ts`**
- Added comprehensive documentation header
- Clarified activation flow
- Documented stack behavior
- Listed MVP limitations with TODO comments
- Cross-referenced main Ability contract

**`src/echomancy/domainmodel/triggers/Trigger.ts`**
- Updated to align with Ability contract
- Documented when triggers are evaluated (specific points only)
- Clarified MVP limitation: triggers execute immediately (not on stack yet)
- Added TODO for proper stack implementation
- Cross-referenced main Ability contract

**`src/echomancy/domainmodel/game/StackTypes.ts`**
- Defined `TriggeredAbilityOnStack` type (not yet used)
- Documented why triggered abilities don't go on stack yet
- Added detailed TODO for implementation
- Updated documentation for `AbilityOnStack`
- Added imports for Game and EffectContext types

**`src/echomancy/domainmodel/effects/EffectContext.ts`**
- Added comprehensive documentation header
- Clarified required vs optional fields
- Documented Last Known Information behavior
- Explained ETB abilities don't reuse spell targets
- Listed MVP limitations with TODO comments

**`src/echomancy/domainmodel/effects/Effect.ts`**
- Added comprehensive documentation header
- Documented implementation rules (must use game.apply(), etc.)
- Clarified execution context
- Listed what effects can and cannot do
- Added MVP limitations section

**`src/echomancy/domainmodel/game/Game.ts`**
- Updated `evaluateTriggers()` documentation
- Clarified when triggers are evaluated (specific points only)
- Added detailed MVP limitations section
- Updated `executeTriggeredAbilities()` documentation
- Added TODO for proper triggered ability stack behavior
- Cross-referenced main Ability contract

## Key Principles

### 1. Abilities are Declarative

Abilities do NOT:
- Subscribe to events
- Maintain listeners
- Execute automatically outside defined evaluation points
- Have their own internal state

Abilities ARE:
- Declarations of "when X happens, do Y"
- Evaluated by the Game at specific points
- Resolved via the Game, never mutating state directly

### 2. Game is the Authority

The Game is the ONLY component that:
- Evaluates triggers
- Activates abilities
- Puts items on the stack
- Resolves abilities

### 3. Evaluation Points are Explicit

Triggered abilities are checked ONLY at:
1. After `enterBattlefield()` → ZONE_CHANGED event
2. After `declareAttacker()` → CREATURE_DECLARED_ATTACKER event
3. After `resolveSpell()` → SPELL_RESOLVED event
4. On step transition → STEP_STARTED, COMBAT_ENDED events

Abilities are NEVER evaluated continuously or reactively.

### 4. No Global Event System

There is NO event bus or publish/subscribe system.
This keeps the engine deterministic and testable.

## MVP Supported

### ActivatedAbility
- Player explicitly activates (requires priority)
- Has an activation cost (currently only {T})
- Goes on the stack as `AbilityOnStack`
- Resolves independently (Last Known Information)

### TriggeredAbility (via Trigger)
- Triggered by specific Game events
- Automatically detected by Game
- **MVP LIMITATION**: Currently executes immediately (does NOT go on stack)
- **TODO**: Should create StackItem for triggered abilities

## MVP NOT Supported

The following are explicitly OUT of scope:

### Ability Types
- StaticAbility (continuous effects)
- ReplacementEffect
- PreventionEffect
- ManaAbility

### Features
- Mana costs (only {T} cost supported)
- Targeting (targets array always empty)
- Timing restrictions ("Activate only during combat")
- Choice-based abilities ("Choose one —")
- "May" abilities (optional)
- Cost reduction
- Trigger ordering control (APNAP not implemented)
- Duration tracking ("until end of turn")
- Delayed triggered abilities
- Intervening-if clauses
- Multiple costs combined
- X costs
- Alternative costs

All non-supported features are marked with TODO comments in the code.

## Critical MVP Limitation

**Triggered abilities execute immediately instead of going on the stack.**

Current behavior:
- Triggers fire when conditions are met
- Effects execute immediately
- No StackItem created
- No priority round for responses
- No APNAP ordering

Future behavior (TODO):
- Create `TriggeredAbilityOnStack` when trigger fires
- Add to stack before priority round
- Resolve via normal stack resolution (LIFO)
- Implement APNAP ordering for simultaneous triggers
- Allow players to respond to triggered abilities

## Implementation Rules

When implementing abilities, ensure:

1. Effects use `game.apply()` for state mutations (never direct mutation)
2. Effects use `game.enterBattlefield()` for permanents (never array.push)
3. Triggers are pure predicates (no side effects in conditions)
4. Abilities do not store mutable state
5. All trigger eventTypes use `GameEventTypes` constants
6. All zone checks use `ZoneNames` constants
7. Effects receive Game and EffectContext, nothing more
8. No direct event subscription or observer pattern

## File Locations

- **Ability contract**: `src/echomancy/domainmodel/abilities/Ability.ts`
- **Activated abilities**: `src/echomancy/domainmodel/abilities/ActivatedAbility.ts`
- **Triggered abilities**: `src/echomancy/domainmodel/triggers/Trigger.ts`
- **Stack types**: `src/echomancy/domainmodel/game/StackTypes.ts`
- **Effects**: `src/echomancy/domainmodel/effects/Effect.ts`
- **Effect context**: `src/echomancy/domainmodel/effects/EffectContext.ts`
- **Effect implementations**: `src/echomancy/domainmodel/effects/impl/`
- **Game logic**: `src/echomancy/domainmodel/game/Game.ts`

## Next Steps

To expand the ability system while preserving these principles:

1. Implement triggered abilities on stack
2. Add targeting support
3. Implement APNAP ordering
4. Add more cost types
5. Implement duration tracking
6. Add static abilities (requires continuous effect system)

Always preserve:
- Game as single source of truth
- Abilities remain declarative (no listeners)
- Stack is the only execution mechanism (except mana abilities)
- Evaluation points remain explicit and deterministic
- No global event bus
