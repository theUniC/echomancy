# Effect System

Effects are the executable part of abilities - what actually happens when an ability resolves.

## Overview

When an ability resolves (from the stack for activated abilities, or immediately for MVP triggers), its Effect executes. Effects receive the game state and an execution context, then use Game methods to produce the desired outcome.

## Effect Interface

All effects implement a single method: `resolve(game, context)`. This method is called when the ability resolves and is responsible for carrying out the effect's action.

## Effect Context

The EffectContext provides all information needed to execute an effect:

- **controllerId**: The player who controls this ability (always present)
- **source**: The card with this ability (may be undefined if it left the battlefield)
- **targets**: Selected targets (always empty in MVP - no targeting system yet)

### Last Known Information

The source field uses Last Known Information semantics. It captures the card state when the ability was activated or triggered. This means:
- The information remains valid even if the source leaves the battlefield
- The source may be undefined if the card no longer exists in any zone
- For triggered abilities, it reflects the state at trigger time

### Important Behaviors

- ETB (enter the battlefield) abilities do not reuse spell targets - the permanent entering is a new object with its own identity
- The controllerId is always present and should be used to identify the controlling player
- Targets are always empty in the MVP since no targeting system is implemented

## Implementation Rules

Effects must follow strict rules to maintain game integrity:

**Must do:**
- Use Game methods for all mutations (drawCards, enterBattlefield, etc.)
- Use context.controllerId to identify the controlling player

**Must not do:**
- Mutate state directly (no pushing to arrays, no property assignment)
- Use game.apply() - that's for player actions, not effect resolution
- Subscribe to events or access external state
- Store instance variables or maintain lifecycle (effects are stateless)

## Effect Location

New effect implementations should be placed in the effects/impl/ directory.

## Available Game Methods

Effects can use these Game methods:
- drawCards(playerId, count) - Draw cards from library
- enterBattlefield(card, controllerId) - Put permanent onto battlefield
- Future: dealDamage, adjustLifeTotal, createToken, etc.

## MVP Limitations

The following effect features are not supported:
- Targeting - the targets array is always empty
- Duration tracking - no "until end of turn" effects
- Modal effects - no "Choose one" abilities
- Damage dealing - no damage system yet
- Token creation - not implemented
- Counter manipulation - not implemented
