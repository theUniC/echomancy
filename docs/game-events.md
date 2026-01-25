# Game Events

Internal data structures representing "something that happened" used to evaluate which triggers should fire.

## Key Concepts

- **Not an Event Bus** - Events are internal to Game, not pub/sub system
- **Synchronous Evaluation** - Triggers evaluated immediately after event occurs
- **Deterministic** - Same state + action always produces same events
- **Five Event Types** - ZONE_CHANGED, STEP_STARTED, CREATURE_DECLARED_ATTACKER, COMBAT_ENDED, SPELL_RESOLVED

## How It Works

### Event Flow

1. Game detects an action occurred
2. Game constructs event object
3. Game evaluates which triggers apply to this event
4. Matching triggers fire

Cards declare triggers with conditions and effects. They do not subscribe to events or maintain internal state.

### Event Types

**ZoneChangedEvent**
Emitted when card moves between zones. Foundation for:
- ETB triggers (entering battlefield from hand/stack)
- Dies triggers (battlefield → graveyard)
- Leaves battlefield triggers (battlefield → any zone)

Includes: card, origin zone, destination zone, controller.

**StepStartedEvent**
Emitted when new step/phase begins. Enables:
- "At the beginning of your upkeep" triggers
- "At the beginning of combat" triggers
- Untap step triggers

Includes: step started, active player.

**CreatureDeclaredAttackerEvent**
Emitted when creature declared as attacker. Enables:
- "Whenever this creature attacks" triggers
- "Whenever a creature you control attacks" triggers

Includes: attacking creature, controller.

**CombatEndedEvent**
Emitted when combat phase ends. Used for:
- Resetting combat states
- "At end of combat" triggers

**SpellResolvedEvent**
Emitted after spell resolves from stack. Fires after effect applied and card moved to final zone. Useful for:
- "Whenever you cast a spell" triggers
- Spell counting effects

### Trigger Evaluation Points

| Game Method | Event Emitted |
|-------------|---------------|
| enterBattlefield() | ZONE_CHANGED |
| declareAttacker() | CREATURE_DECLARED_ATTACKER |
| resolveSpell() | SPELL_RESOLVED |
| advanceStep() | STEP_STARTED |
| endCombat() | COMBAT_ENDED |

See `src/domainmodel/game/game.ts` for implementation.

### Event Type Constants

GameEventTypes object provides constants: ZONE_CHANGED, STEP_STARTED, CREATURE_DECLARED_ATTACKER, COMBAT_ENDED, SPELL_RESOLVED.

Use these instead of string literals. Located in `src/echomancy/domainmodel/game/GameEvents.ts`.

## Rules

- Events are internal to Game (no external subscribers)
- Triggers evaluated synchronously after event occurs
- Same game state + action = same events (deterministic)
- Cards declare triggers, don't subscribe to events
- Use GameEventTypes constants, never string literals
