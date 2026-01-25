# Creature Stats System

The creature stats system models power, toughness, and counters for creatures on the battlefield.

## Key Concepts

- **Base Power/Toughness** - Static values from the card definition
- **Counters** - Persistent modifications (currently only +1/+1 counters)
- **Current Power/Toughness** - Calculated value (base + counters)
- **Damage Tracking** - Separate from toughness (cleared each turn)

## How It Works

See `src/echomancy/domainmodel/game/Game.ts` for implementation details.

**Initialization**: Stats are set when a creature enters the battlefield via `initializeCreatureStateIfNeeded()`. Values come from the card definition. Defaults: power = 0, toughness = 1.

**Calculation**: Current power/toughness = base value + counters. Calculated on demand, no caching.

**Counter Operations**:
- Add counters: amount must be positive, counters accumulate
- Remove counters: amount must be positive, clamped at 0 (can't go negative)
- Query counters: returns count or 0 if none exist

**Lifecycle**: Stats initialize on ETB, persist while on battlefield, deleted when creature changes zones.

## Rules

- Power and toughness are independent from damage
- Counter amounts must be positive (> 0)
- Counter counts are always >= 0
- Calculations are pure (no side effects)
- Stats don't depend on turn, phase, or stack state
- Counters are lost when creatures change zones

**Testing**: See `Game.creatureStats.test.ts` for comprehensive test coverage.
