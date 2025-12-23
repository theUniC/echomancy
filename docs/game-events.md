# Game Events

Game events represent "something that happened" in the game. They are used to evaluate which triggers should fire.

## Conceptual Model

Events are not an event bus or observable system. They are data structures that the Game uses internally. When something happens:

1. Game detects that an action occurred
2. Game constructs an event object
3. Game inspects the current game state
4. Game evaluates which triggers apply to this event

Cards declare triggers with conditions and effects. They do not subscribe to events or maintain internal state.

## Event Types

### ZoneChangedEvent

Emitted when a card moves from one zone to another. This is the foundation for:
- ETB triggers (entering the battlefield from hand or stack)
- Dies triggers (moving from battlefield to graveyard)
- Leaves battlefield triggers (moving from battlefield to any other zone)

The event includes the card that moved, the origin zone, the destination zone, and the controller.

### StepStartedEvent

Emitted when a new step or phase begins. This enables:
- "At the beginning of your upkeep" triggers
- "At the beginning of combat" triggers
- Untap step triggers

The event includes which step started and the active player.

### CreatureDeclaredAttackerEvent

Emitted when a creature is declared as an attacker. This enables:
- "Whenever this creature attacks" triggers
- "Whenever a creature you control attacks" triggers

The event includes the attacking creature and its controller.

### CombatEndedEvent

Emitted when the combat phase ends. Used for:
- Resetting combat-related states
- "At end of combat" triggers

### SpellResolvedEvent

Emitted after a spell finishes resolving from the stack. This fires after the spell's effect has been applied and the card has been moved to its final zone. Useful for:
- "Whenever you cast a spell" triggers (post-resolution hooks)
- Spell counting effects

## Event Type Constants

The GameEventTypes object provides constants for all event types: ZONE_CHANGED, STEP_STARTED, CREATURE_DECLARED_ATTACKER, COMBAT_ENDED, and SPELL_RESOLVED. Use these instead of string literals.

## Trigger Evaluation Points

Events are emitted at specific points in the Game:

| Game Method | Event Emitted |
|-------------|---------------|
| enterBattlefield() | ZONE_CHANGED |
| declareAttacker() | CREATURE_DECLARED_ATTACKER |
| resolveSpell() | SPELL_RESOLVED |
| advanceStep() | STEP_STARTED |
| endCombat() | COMBAT_ENDED |

## Design Notes

**No Event Bus:** There is no pub/sub system. Events are internal to the Game.

**Synchronous Evaluation:** Triggers are evaluated synchronously after the event occurs.

**Deterministic:** The same game state plus action always produces the same events and trigger evaluations.

**MVP Limitation:** Triggered abilities currently execute immediately rather than going on the stack. Players cannot respond to triggers in the MVP.
