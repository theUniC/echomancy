# Architecture

This document describes the core architectural principles and design decisions of the Echomancy game engine.

## Design Philosophy

Echomancy follows **Domain-Driven Design (DDD)** principles, modeling Magic: The Gathering's rules as a rich domain model. The engine prioritizes:

1. **Correctness** over performance
2. **Explicitness** over convenience
3. **Testability** over flexibility
4. **Type safety** throughout

## Core Principles

### 1. Abilities are Declarative

Abilities do **NOT**:
- Subscribe to events
- Maintain listeners
- Execute automatically outside defined evaluation points
- Have their own internal state

Abilities **ARE**:
- Declarations of "when X happens, do Y"
- Evaluated by the Game at specific points
- Resolved via the Game, never mutating state directly

```typescript
// CORRECT: Declarative trigger definition
const trigger: Trigger = {
  eventType: GameEventTypes.ZONE_CHANGED,
  condition: (game, event, source) =>
    event.card.instanceId === source.instanceId &&
    event.toZone === ZoneNames.BATTLEFIELD,
  effect: (game, context) => game.drawCards(context.controllerId, 1)
}

// WRONG: Active listener (not supported)
card.on('enterBattlefield', () => game.drawCards(...))
```

### 2. Game is the Single Authority

The `Game` class is the **ONLY** component that:
- Evaluates triggers
- Activates abilities
- Puts items on the stack
- Resolves abilities
- Mutates game state

All state changes flow through `game.apply()` for player actions or specific Game methods like `drawCards()`, `enterBattlefield()` for effect execution.

### 3. Evaluation Points are Explicit

Triggered abilities are checked **ONLY** at specific points:

| Method | Event Emitted |
|--------|---------------|
| `enterBattlefield()` | `ZONE_CHANGED` |
| `declareAttacker()` | `CREATURE_DECLARED_ATTACKER` |
| `resolveSpell()` | `SPELL_RESOLVED` |
| Step transitions | `STEP_STARTED`, `COMBAT_ENDED` |

Abilities are **NEVER** evaluated continuously or reactively.

### 4. No Global Event System

There is **NO** event bus or publish/subscribe system. Events are data structures that the Game uses to evaluate triggers, not messages to be consumed by subscribers.

This keeps the engine:
- Deterministic
- Testable
- Easy to reason about
- Free of hidden dependencies

## Command Pattern

All player actions use the command pattern via `game.apply()`:

```typescript
// Player actions go through apply()
game.apply({ type: "PLAY_LAND", playerId, cardId })
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })
game.apply({ type: "PASS_PRIORITY", playerId })
game.apply({ type: "DECLARE_ATTACKER", playerId, creatureId })
```

This ensures:
- All actions are validated before execution
- Game rules are enforced consistently
- State changes are traceable

## State Management

### Immutable Queries, Controlled Mutations

- Query methods return readonly views of state
- Mutations only happen through defined Game methods
- Effects use Game methods, never direct state mutation

```typescript
// CORRECT: Use Game methods
game.drawCards(playerId, 1)
game.enterBattlefield(permanent, controllerId)

// WRONG: Direct mutation
playerState.hand.cards.push(card)  // Bypasses ETB triggers!
```

### Player State Structure

Each player has isolated zones:

```typescript
type PlayerState = {
  hand: Zone
  battlefield: Zone
  graveyard: Zone
}
```

## Error Handling

Domain-specific errors extend `GameError`:

| Error | When Thrown |
|-------|-------------|
| `InvalidPlayerCountError` | Game started with < 2 players |
| `InvalidPlayLandStepError` | Land played outside main phase |
| `LandLimitExceededError` | Second land played in a turn |
| `CardNotFoundInHandError` | Card not in player's hand |
| `TappedCreatureCannotAttackError` | Tapped creature declared as attacker |

See `src/echomancy/domainmodel/game/GameErrors.ts` for the complete list.

## MVP Limitations

The following features are explicitly **OUT OF SCOPE** for the MVP:

### Ability Types Not Supported
- StaticAbility (continuous effects)
- ReplacementEffect
- PreventionEffect
- ManaAbility

### Features Not Supported
- Mana costs (only {T} tap cost)
- Targeting (targets array always empty)
- Timing restrictions
- Choice-based abilities
- Optional ("may") abilities
- Duration tracking ("until end of turn")
- APNAP ordering
- Delayed triggered abilities
- Intervening-if clauses

### Critical MVP Limitation

**Triggered abilities execute immediately instead of going on the stack.**

Current behavior:
- Triggers fire when conditions are met
- Effects execute immediately
- No StackItem created
- No priority round for responses

Future behavior (TODO):
- Create `TriggeredAbilityOnStack` when trigger fires
- Add to stack before priority round
- Resolve via normal stack resolution (LIFO)
- Implement APNAP ordering for simultaneous triggers

## File Organization

```
src/echomancy/domainmodel/
├── abilities/
│   ├── Ability.ts           # Union type and type guards
│   └── ActivatedAbility.ts  # Activated ability definition
├── cards/
│   ├── CardDefinition.ts    # Card template with types, triggers
│   └── CardInstance.ts      # Runtime card with unique ID
├── effects/
│   ├── Effect.ts            # Effect interface
│   ├── EffectContext.ts     # Execution context
│   └── impl/                # Concrete implementations
├── game/
│   ├── Game.ts              # Core engine
│   ├── GameActions.ts       # Action types
│   ├── GameErrors.ts        # Error classes
│   ├── GameEvents.ts        # Event types
│   ├── Player.ts            # Player model
│   ├── PlayerState.ts       # Zone-based state
│   ├── StackTypes.ts        # Stack item types
│   ├── Steps.ts             # Turn phases
│   └── StepMachine.ts       # Phase transitions
├── targets/
│   └── Target.ts            # Target types (future)
├── triggers/
│   └── Trigger.ts           # Trigger definition
└── zones/
    └── Zone.ts              # Zone types and constants
```

## Next Steps

To expand the engine while preserving these principles:

1. Implement triggered abilities on stack
2. Add targeting support
3. Implement APNAP ordering
4. Add more cost types (mana, sacrifice, etc.)
5. Implement duration tracking
6. Add static abilities (requires continuous effect system)

Always preserve:
- Game as single source of truth
- Abilities remain declarative (no listeners)
- Stack is the only execution mechanism (except mana abilities)
- Evaluation points remain explicit and deterministic
- No global event bus
