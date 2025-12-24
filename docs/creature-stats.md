# Creature Stats System

This document describes the power/toughness and counter system for creatures in Echomancy.

## Overview

The creature stats system models the numeric identity of creatures:
- Base power and toughness
- Counters that modify these values
- Calculation of current power and toughness

This is a foundational system required for combat damage, creature destruction, and combat-related UI.

## Base Power and Toughness

Every creature has base power and toughness values defined in its card definition:

- **Base Power**: The creature's inherent offensive strength
- **Base Toughness**: The creature's inherent resilience

These values are set when the creature enters the battlefield and remain constant unless specifically changed by effects that modify base characteristics (not implemented in MVP).

### Default Values

When a creature card doesn't specify power/toughness:
- Power defaults to `0`
- Toughness defaults to `1` (minimum viable creature)

### Initialization

Creature stats are initialized when a permanent enters the battlefield through the `initializeCreatureStateIfNeeded` method. The values come from the card's definition.

## Counters

Counters are persistent modifications that remain on a creature. They're different from temporary effects or static abilities.

### Counter Types

MVP supports:
- **+1/+1 counters**: Increase both power and toughness by 1 for each counter

Future expansion will include:
- -1/-1 counters
- Poison counters
- Charge counters
- Other counter types as needed

### Counter Operations

**Adding Counters**:
- Amount must be positive (> 0)
- Counters accumulate additively
- Multiple additions stack

**Removing Counters**:
- Amount must be positive (> 0)
- Counter count cannot go below 0 (clamped)
- Removing more counters than exist is safe (results in 0)

**Querying Counters**:
- Returns the current count for a specific counter type
- Returns 0 if no counters of that type exist

## Current Power and Toughness

Current values are calculated from base values plus counters:

```
Current Power = Base Power + (+1/+1 counters)
Current Toughness = Base Toughness + (+1/+1 counters)
```

### Calculation Rules

- Calculations are deterministic and pure (no side effects)
- Only +1/+1 counters affect the calculation in MVP
- Values are recalculated on every query
- No caching or memoization in MVP

## API Methods

The Game class provides these methods for creature stats:

**Query Methods** (read-only):
- `getBasePower(creatureId)`: Get base power value
- `getBaseToughness(creatureId)`: Get base toughness value
- `getCounters(creatureId, counterType)`: Get counter count
- `getCurrentPower(creatureId)`: Calculate current power
- `getCurrentToughness(creatureId)`: Calculate current toughness

**Mutation Methods**:
- `addCounters(creatureId, counterType, amount)`: Add counters to a creature
- `removeCounters(creatureId, counterType, amount)`: Remove counters from a creature

All methods throw errors if the creature doesn't exist.

## Lifecycle

**Initialization**:
- Creature stats are initialized when entering the battlefield
- Values come from the card definition
- Counters start at 0 for all types

**During Gameplay**:
- Base values remain constant (unless explicitly changed by effects)
- Counters can be added or removed
- Current values are calculated on demand

**Cleanup**:
- Creature stats are deleted when the creature leaves the battlefield
- Counters are lost when a creature changes zones

## MVP Limitations

The following are intentionally excluded from the MVP:

**Not Supported**:
- Damage tracking (separate from toughness)
- Static ability modifiers (lords, anthems)
- Continuous effects layers
- Temporary "until end of turn" modifications
- -1/-1 counters and counter interaction rules
- Effects that set base power/toughness
- Power/toughness-switching effects

**Future Implementation**:
- Damage model (damage marked on creatures)
- Lethal damage checks
- State-based actions for creature destruction
- 7-layer system for continuous effects
- Static abilities that modify power/toughness

## Invariants

The system maintains these invariants:

1. Power and toughness are independent from damage
2. Counter removal never corrupts base values
3. Calculations are pure (no side effects)
4. Identical creatures produce identical results
5. The system doesn't depend on turn, phase, or stack state
6. Counter counts are always >= 0

## Testing

Comprehensive tests cover:
- Base power/toughness initialization
- Counter addition and removal
- Current power/toughness calculation
- Edge cases (0/0 creatures, large stats, negative operations)
- Multi-creature independence
- Invariant validation

See `Game.creatureStats.test.ts` for the full test suite.

## Integration with Other Systems

**Combat System** (planned):
- Will query current power for damage assignment
- Will query current toughness for lethal damage checks
- Does not modify stats directly

**Continuous Effects** (planned):
- Will modify current power/toughness through layer system
- Will not modify base values or counters directly

**State-Based Actions** (planned):
- Will check current toughness vs damage for creature destruction
- Will use current power/toughness for 0-toughness checks

## Design Principles

This system follows Echomancy's core principles:

1. **Correctness over completeness**: MVP includes only +1/+1 counters
2. **Explicit over implicit**: All calculations are explicit method calls
3. **Stateless where possible**: Calculations have no side effects
4. **Type-safe**: All values are strongly typed
5. **Testable**: Pure functions make testing straightforward
6. **Documented limitations**: All excluded features have TODO markers
