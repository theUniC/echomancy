# Architecture

Domain-Driven Design (DDD) approach to modeling Magic: The Gathering rules as a rich domain model.

## Key Concepts

- **Game as Aggregate Root** - Single source of truth for all game state
- **Abilities are Declarative** - Cards declare behavior; Game evaluates at explicit points
- **Command Pattern** - All actions through `game.apply()`
- **Value Objects** - Immutable state (ManaPool, PermanentState, TurnState)
- **Domain Services** - Stateless operations (CombatResolution, TriggerEvaluation)
- **Specifications** - Composable business rules (CanPlayLand, CanCastSpell)

## How It Works

### Design Priorities

1. **Correctness** over performance
2. **Explicitness** over convenience
3. **Testability** over flexibility
4. **Type safety** throughout

### Game is the Single Authority

The Game class is the only component that evaluates triggers, activates abilities, puts items on the stack, resolves abilities, and mutates game state. All state changes flow through Game methods.

**Critical rule**: Always use `game.enterBattlefield()` to put permanents on the battlefield, never push directly to arrays. Direct manipulation bypasses ETB trigger system.

### Evaluation Points are Explicit

Triggered abilities are checked only at specific points after game actions:

| Game Action | Event Produced |
|-------------|----------------|
| Permanent enters battlefield | ZONE_CHANGED |
| Creature declared as attacker | CREATURE_DECLARED_ATTACKER |
| Spell resolves | SPELL_RESOLVED |
| Step/phase transitions | STEP_STARTED, COMBAT_ENDED |

Abilities are never evaluated continuously or reactively. No global event bus exists.

### DDD Building Blocks

Implementation organized by DDD patterns:

**Value Objects** (`domainmodel/game/valueobjects/`)
- ManaPool - Immutable mana management
- PermanentState - Permanent state (tap, counters, creature stats)
- TurnState - Turn tracking (player, step, turn number)
- CombatState - Attacker/blocker assignments

**Entities** (`domainmodel/game/entities/`)
- Battlefield, Hand, Graveyard - Zone management
- TheStack - LIFO stack for spells/abilities

**Domain Services** (`domainmodel/game/services/`)
- CombatResolution - Damage calculation
- TriggerEvaluation - Match triggers to events
- StateBasedActions - Identify creatures to destroy

**Specifications** (`domainmodel/game/specifications/`)
- CanPlayLand, CanCastSpell - Validation rules
- CanDeclareAttacker, CanDeclareBlocker - Combat rules
- CanActivateAbility, HasPriority - Activation rules

### State Management

- **Immutable queries** - Query methods return readonly views
- **Controlled mutations** - Only through Game methods
- **Player isolation** - Each player has isolated zones (hand, battlefield, graveyard)

### Error Handling

Domain errors extend GameError class. Each type corresponds to specific rule violations (invalid step, resource limits, card not found, state violations).

See `src/domainmodel/game/errors/` for implementation.

## Rules

- Game is the only entity that mutates state
- Abilities are data structures, not active code
- All player actions use command pattern via `game.apply()`
- Effects use Game methods (`drawCards()`, `enterBattlefield()`), never direct state mutation
- Evaluation points are explicit and deterministic
- No global event system or pub/sub

