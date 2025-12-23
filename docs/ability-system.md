# Ability System

The ability system defines how cards produce effects through activated and triggered abilities.

## Overview

An **Ability** is a rule unit attached to a card that produces effects when specific conditions are met. Abilities are **DECLARATIVE** - they don't execute actively, but are evaluated by the Game at specific points.

```typescript
type Ability = ActivatedAbility | Trigger
```

## Supported Ability Types

### Activated Abilities

Player-activated abilities with costs that go on the stack.

**Example**: `{T}: Draw a card`

```typescript
type ActivatedAbility = {
  cost: ActivationCost
  effect: Effect
}
```

**Activation Flow**:
1. Player has priority
2. Player pays the cost (currently only tap)
3. `AbilityOnStack` is created
4. Opponents get priority to respond
5. Ability resolves (LIFO)

**MVP Limitations**:
- Only `{T}` (tap) cost supported
- No mana costs, sacrifice, discard, or life payment
- No targeting
- No timing restrictions (can activate any time with priority)

### Triggered Abilities

Event-based abilities that fire automatically when conditions are met.

**Example**: `When this creature attacks, draw a card`

```typescript
type Trigger = {
  eventType: GameEvent["type"]
  condition: (game: Game, event: GameEvent, sourceCard: CardInstance) => boolean
  effect: (game: Game, context: EffectContext) => void
}
```

**Evaluation Flow**:
1. Game detects an event occurred
2. Game constructs event object
3. Game evaluates all triggers on all permanents
4. Matching triggers execute their effects

**MVP Limitations**:
- Triggers execute immediately (don't go on stack yet)
- No targeting
- No optional triggers
- No intervening-if clauses
- No APNAP ordering

## Type Guards

Use type guards to distinguish ability types at runtime:

```typescript
import { isActivatedAbility, isTrigger } from "@/echomancy/domainmodel/abilities/Ability"

if (isActivatedAbility(ability)) {
  // ability.cost, ability.effect available
}

if (isTrigger(ability)) {
  // ability.eventType, ability.condition, ability.effect available
}
```

## Activation Costs

Currently only tap cost is supported:

```typescript
type ActivationCost = {
  type: "TAP"
}
```

**Future cost types** (TODO):
- Mana costs
- Sacrifice costs
- Discard costs
- Life payment
- Multiple combined costs
- X costs

## Defining Triggers

### ETB Trigger (Enter the Battlefield)

```typescript
import { GameEventTypes } from "@/echomancy/domainmodel/game/GameEvents"
import { ZoneNames } from "@/echomancy/domainmodel/zones/Zone"

const etbTrigger: Trigger = {
  eventType: GameEventTypes.ZONE_CHANGED,
  condition: (game, event, source) =>
    event.card.instanceId === source.instanceId &&
    event.toZone === ZoneNames.BATTLEFIELD,
  effect: (game, context) =>
    game.drawCards(context.controllerId, 1)
}
```

### Attack Trigger

```typescript
const attackTrigger: Trigger = {
  eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
  condition: (game, event, source) =>
    event.creature.instanceId === source.instanceId,
  effect: (game, context) =>
    game.drawCards(context.controllerId, 1)
}
```

### Conditional Trigger

```typescript
// "When this enters, if you control another Elf, draw a card"
const conditionalTrigger: Trigger = {
  eventType: GameEventTypes.ZONE_CHANGED,
  condition: (game, event, source) => {
    if (event.card.instanceId !== source.instanceId) return false
    if (event.toZone !== ZoneNames.BATTLEFIELD) return false

    // Check for another elf
    const battlefield = game.getPlayerState(event.controllerId).battlefield.cards
    return battlefield.some(card =>
      card.instanceId !== source.instanceId &&
      card.definition.name.toLowerCase().includes("elf")
    )
  },
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}
```

## Helper Type for Better Inference

Use `TriggerDefinition<T>` for better type inference on the event parameter:

```typescript
import type { TriggerDefinition } from "@/echomancy/domainmodel/triggers/Trigger"

const etbTrigger: TriggerDefinition<"ZONE_CHANGED"> = {
  eventType: "ZONE_CHANGED",
  condition: (game, event, source) => {
    // event is typed as ZoneChangedEvent
    return event.toZone === "BATTLEFIELD"
  },
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}
```

## Adding Abilities to Cards

### Card with Activated Ability

```typescript
const cardWithTapAbility: CardDefinition = {
  id: "tap-to-draw",
  name: "Card That Draws",
  types: ["CREATURE"],
  activatedAbility: {
    cost: { type: "TAP" },
    effect: new DrawCardsEffect(1)
  }
}
```

### Card with Trigger

```typescript
const cardWithETB: CardDefinition = {
  id: "elvish-visionary",
  name: "Elvish Visionary",
  types: ["CREATURE"],
  triggers: [{
    eventType: GameEventTypes.ZONE_CHANGED,
    condition: (game, event, source) =>
      event.card.instanceId === source.instanceId &&
      event.toZone === ZoneNames.BATTLEFIELD,
    effect: (game, context) => game.drawCards(context.controllerId, 1)
  }]
}
```

## Implementation Rules

When implementing abilities:

1. **Effects use Game methods** for mutations (`drawCards`, `enterBattlefield`, etc.)
2. **Never use `game.apply()`** in effects - that's for player actions
3. **Triggers are pure predicates** - no side effects in conditions
4. **Abilities don't store mutable state**
5. **Use constants** - `GameEventTypes` and `ZoneNames` to avoid magic strings
6. **Effects receive only** `Game` and `EffectContext`

## Not Supported (Out of MVP Scope)

The following ability types are not implemented:

| Type | Description |
|------|-------------|
| StaticAbility | Continuous effects while on battlefield |
| ReplacementEffect | "If X would happen, instead Y" |
| PreventionEffect | "Prevent the next N damage" |
| ManaAbility | Abilities that produce mana (don't use stack) |

## Source Files

| File | Purpose |
|------|---------|
| `abilities/Ability.ts` | Union type and type guards |
| `abilities/ActivatedAbility.ts` | Activated ability and cost types |
| `triggers/Trigger.ts` | Trigger type and helper types |
| `game/Game.ts` | Evaluation and execution logic |
