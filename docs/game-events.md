# Game Events

Game events represent "something that happened" in the game. They are used to evaluate which triggers should fire.

## Conceptual Model

Events are **NOT** an event bus or observable system. They are data structures that the Game uses internally to:

1. Detect that something happened
2. Construct an event object
3. Inspect the current game state
4. Evaluate which triggers apply

Cards declare triggers (conditions + effects), they do **NOT** subscribe to events or maintain internal state.

## Event Types

### ZoneChangedEvent

Emitted when a card moves from one zone to another.

```typescript
type ZoneChangedEvent = {
  type: "ZONE_CHANGED"
  card: CardInstance
  fromZone: ZoneName
  toZone: ZoneName
  controllerId: string
}
```

**Use cases**:
- ETB triggers (hand/stack → battlefield)
- Dies triggers (battlefield → graveyard)
- Leaves battlefield triggers (battlefield → any other zone)

**Example trigger**:
```typescript
{
  eventType: GameEventTypes.ZONE_CHANGED,
  condition: (game, event, source) =>
    event.card.instanceId === source.instanceId &&
    event.toZone === ZoneNames.BATTLEFIELD,
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}
```

### StepStartedEvent

Emitted when a new step/phase begins.

```typescript
type StepStartedEvent = {
  type: "STEP_STARTED"
  step: GameSteps
  activePlayerId: string
}
```

**Use cases**:
- "At the beginning of your upkeep..." triggers
- "At the beginning of combat..." triggers
- Untap triggers

**Example trigger**:
```typescript
{
  eventType: GameEventTypes.STEP_STARTED,
  condition: (game, event, source) =>
    event.step === Step.UPKEEP &&
    event.activePlayerId === source.ownerId,
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}
```

### CreatureDeclaredAttackerEvent

Emitted when a creature is declared as an attacker.

```typescript
type CreatureDeclaredAttackerEvent = {
  type: "CREATURE_DECLARED_ATTACKER"
  creature: CardInstance
  controllerId: string
}
```

**Use cases**:
- "Whenever this creature attacks..." triggers
- "Whenever a creature you control attacks..." triggers

**Example trigger**:
```typescript
{
  eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
  condition: (game, event, source) =>
    event.creature.instanceId === source.instanceId,
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}
```

### CombatEndedEvent

Emitted when the combat phase ends.

```typescript
type CombatEndedEvent = {
  type: "COMBAT_ENDED"
  activePlayerId: string
}
```

**Use cases**:
- Reset of combat-related states
- "At end of combat..." triggers

### SpellResolvedEvent

Emitted after a spell finishes resolving from the stack.

```typescript
type SpellResolvedEvent = {
  type: "SPELL_RESOLVED"
  card: CardInstance
  controllerId: string
}
```

**Note**: This fires AFTER the spell's effect has been applied and the card has been moved to its final zone.

**Use cases**:
- "Whenever you cast a spell..." triggers (post-resolution hooks)
- Spell counting effects

## Event Type Constants

Use `GameEventTypes` constants to avoid magic strings:

```typescript
import { GameEventTypes } from "@/echomancy/domainmodel/game/GameEvents"

// CORRECT
eventType: GameEventTypes.ZONE_CHANGED

// AVOID
eventType: "ZONE_CHANGED"
```

Available constants:

| Constant | Value |
|----------|-------|
| `GameEventTypes.ZONE_CHANGED` | `"ZONE_CHANGED"` |
| `GameEventTypes.STEP_STARTED` | `"STEP_STARTED"` |
| `GameEventTypes.CREATURE_DECLARED_ATTACKER` | `"CREATURE_DECLARED_ATTACKER"` |
| `GameEventTypes.COMBAT_ENDED` | `"COMBAT_ENDED"` |
| `GameEventTypes.SPELL_RESOLVED` | `"SPELL_RESOLVED"` |

## Trigger Evaluation Points

Events are emitted at specific points in the Game:

| Game Method | Event Emitted |
|-------------|---------------|
| `enterBattlefield()` | `ZONE_CHANGED` |
| `declareAttacker()` | `CREATURE_DECLARED_ATTACKER` |
| `resolveSpell()` | `SPELL_RESOLVED` |
| `advanceStep()` | `STEP_STARTED` |
| `endCombat()` | `COMBAT_ENDED` |

## Union Type

All events are part of a discriminated union:

```typescript
type GameEvent =
  | ZoneChangedEvent
  | StepStartedEvent
  | CreatureDeclaredAttackerEvent
  | CombatEndedEvent
  | SpellResolvedEvent
```

This allows type-safe pattern matching:

```typescript
function handleEvent(event: GameEvent) {
  switch (event.type) {
    case "ZONE_CHANGED":
      // event is ZoneChangedEvent
      console.log(`${event.card.definition.name} moved to ${event.toZone}`)
      break
    case "CREATURE_DECLARED_ATTACKER":
      // event is CreatureDeclaredAttackerEvent
      console.log(`${event.creature.definition.name} attacks!`)
      break
    // ... other cases
  }
}
```

## Important Design Notes

1. **No Event Bus**: There is no pub/sub system. Events are internal to the Game.

2. **Synchronous Evaluation**: Triggers are evaluated synchronously after the event occurs.

3. **MVP Limitation**: Triggered abilities currently execute immediately rather than going on the stack.

4. **Deterministic**: The same game state + action always produces the same events and trigger evaluations.

## Source Files

| File | Purpose |
|------|---------|
| `game/GameEvents.ts` | Event types and constants |
| `game/Game.ts` | Event emission and trigger evaluation |
