# Ability System

The ability system defines how cards produce effects through activated and triggered abilities.

## Overview

An Ability is a rule unit attached to a card that produces effects when specific conditions are met. Abilities are declarative - they describe what should happen, not how to make it happen. The Game evaluates them at specific points.

## Ability Types

### Activated Abilities

Activated abilities are player-initiated. A player with priority pays a cost and the ability goes on the stack. When it resolves, the effect executes.

The activation flow:
1. Player has priority
2. Player pays the cost (in MVP, only tap cost is supported)
3. An ability item is created and put on the stack
4. Other players get priority to respond
5. When the stack resolves, the effect executes

Activated abilities use "Last Known Information" - if the source permanent leaves the battlefield before resolution, the ability still resolves using the information captured at activation time.

### Triggered Abilities

Triggered abilities fire automatically when their trigger condition is met. Unlike activated abilities, the player doesn't choose when they happen.

The trigger flow:
1. A game event occurs (creature enters battlefield, attacks, etc.)
2. Game evaluates all triggers on all permanents
3. Triggers whose conditions match the event fire
4. The trigger's effect executes

**MVP Limitation:** Currently, triggered abilities execute immediately when they fire. In the full rules, they should go on the stack and players can respond. This simplification will be addressed in a future update.

## Trigger Conditions

A trigger consists of three parts:
- **Event type**: What kind of game event activates this trigger
- **Condition**: A predicate that must be true for the trigger to fire
- **Effect**: What happens when the trigger fires

The condition is a pure predicate with no side effects. It receives the game state, the event that occurred, and the source card, then returns true or false.

## Type Guards

Two type guard functions distinguish ability types at runtime: `isActivatedAbility()` checks for cost and effect properties, while `isTrigger()` checks for eventType and condition properties.

## Activation Costs

The MVP supports only tap cost. Future cost types planned:
- Mana costs
- Sacrifice costs
- Discard costs
- Life payment
- Combined costs
- X costs

## Adding Abilities to Cards

Cards can have:
- An `activatedAbility` property for a single activated ability
- A `triggers` array for multiple triggered abilities

The MVP doesn't support cards with multiple activated abilities.

## Implementation Rules

When implementing abilities:
- Effects must use Game methods for all state mutations
- Never use `game.apply()` inside an effect (that's for player actions)
- Trigger conditions must be pure functions with no side effects
- Abilities don't store mutable state
- Use the provided constants (GameEventTypes, ZoneNames) instead of string literals

## Not Supported in MVP

The following ability types are out of scope:
- Static abilities (continuous effects while on battlefield)
- Replacement effects ("If X would happen, instead Y")
- Prevention effects ("Prevent the next N damage")
- Mana abilities (abilities that produce mana and don't use the stack)
