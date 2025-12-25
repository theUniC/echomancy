# Architecture

This document describes the core architectural principles and design decisions of the Echomancy game engine.

## Design Philosophy

Echomancy follows Domain-Driven Design (DDD) principles, modeling Magic: The Gathering's rules as a rich domain model. The engine prioritizes:

1. **Correctness** over performance
2. **Explicitness** over convenience
3. **Testability** over flexibility
4. **Type safety** throughout

## Core Principles

### 1. Abilities are Declarative

Abilities are declarations of "when X happens, do Y". They do not subscribe to events, maintain listeners, or execute automatically outside defined evaluation points. Cards declare what their abilities do; the Game evaluates them at specific points.

This means abilities are data structures describing behavior, not active code that runs on its own.

### 2. Game is the Single Authority

The Game class is the only component that:
- Evaluates triggers
- Activates abilities
- Puts items on the stack
- Resolves abilities
- Mutates game state

All state changes flow through the Game. Player actions go through `game.apply()`, and effect execution uses specific Game methods like `drawCards()` or `enterBattlefield()`.

### 3. Evaluation Points are Explicit

Triggered abilities are checked only at specific points:

| Game Action | Event Produced |
|-------------|----------------|
| Permanent enters battlefield | ZONE_CHANGED |
| Creature declared as attacker | CREATURE_DECLARED_ATTACKER |
| Spell resolves | SPELL_RESOLVED |
| Step/phase transitions | STEP_STARTED, COMBAT_ENDED |

Abilities are never evaluated continuously or reactively. The Game explicitly checks for triggers after each action that could produce an event.

### 4. No Global Event System

There is no event bus or publish/subscribe system. Events are data structures that the Game uses internally to evaluate triggers, not messages to be consumed by external subscribers.

This keeps the engine deterministic, testable, and free of hidden dependencies.

## Command Pattern

All player actions use the command pattern via `game.apply()`. This ensures:
- All actions are validated before execution
- Game rules are enforced consistently
- State changes are traceable

## State Management

### Immutable Queries, Controlled Mutations

Query methods return readonly views of state. Mutations only happen through defined Game methods. Effects use Game methods, never direct state mutation.

The critical rule: always use `game.enterBattlefield()` to put permanents on the battlefield, never push directly to the battlefield array. Direct manipulation bypasses the ETB (enter the battlefield) trigger system.

### Player State Structure

Each player has isolated zones: hand, battlefield, and graveyard. The library and exile zones also exist but are less frequently accessed in the MVP.

## Error Handling

Domain-specific errors extend a base GameError class. Each error type corresponds to a specific rule violation:

- Invalid step for an action (playing land outside main phase)
- Resource limits exceeded (second land in a turn)
- Card not found in expected zone
- Creature state violations (tapped creature attacking)

## MVP Limitations

The following are explicitly out of scope for the MVP:

**Ability Types Not Supported:**
- Continuous effects / lords (static abilities that affect other permanents)
- Replacement effects
- Prevention effects
- Mana abilities

**Static Abilities (Partial Support):**
- ✅ Consultative keywords implemented (Flying, Reach, Vigilance)
- ❌ Full 7-layer system (deferred)
- ❌ Ability gain/loss ("creature gains flying until end of turn")
- ❌ Continuous effects ("other creatures get +1/+1")

See `docs/static-abilities.md` for the consultative keywords implementation.

**Features Not Supported:**
- Mana costs (mana pool exists but cost evaluation not implemented)
- Targeting system (basic targeting for spells exists but incomplete)
- Duration tracking ("until end of turn" effects)
- APNAP ordering for simultaneous triggers
- Choice-based or optional abilities

**Critical MVP Limitation:**
Triggered abilities currently execute immediately instead of going on the stack. In the full rules, triggered abilities should be put on the stack and players can respond to them. This is a simplification for the MVP.

## Future Direction

To expand the engine while preserving these principles:

1. Implement triggered abilities on stack
2. Add targeting support
3. Implement APNAP ordering
4. Add more cost types (mana, sacrifice, etc.)
5. Implement duration tracking
6. Implement full 7-layer system for static abilities
7. Add continuous effects and lords

The key invariants to preserve:
- Game as single source of truth
- Abilities remain declarative
- Stack is the only execution mechanism (except mana abilities)
- Evaluation points remain explicit and deterministic
