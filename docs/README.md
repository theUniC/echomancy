# Echomancy Documentation

Echomancy is a Magic: The Gathering game engine built with Domain-Driven Design principles in TypeScript.

## Documentation Index

### Core Concepts

| Document | Description |
|----------|-------------|
| [Architecture](./architecture.md) | Core design principles and architectural patterns |
| [UI Architecture](./ui-architecture.md) | How UI interacts with the game engine |
| [API Conventions](./api-conventions.md) | RESTful API design and route handlers |
| [Commands and Queries](./commands-and-queries.md) | CQRS-lite pattern for application layer |
| [Turn Structure](./turn-structure.md) | Game phases, steps, and turn progression |
| [Zones and Cards](./zones-and-cards.md) | Game zones, card definitions, and instances |
| [Game State Export](./game-state-export.md) | Complete state export for UI and external consumers |
| [Game Snapshot](./game-snapshot.md) | UI-facing game state with visibility filtering and player perspective |

### Systems

| Document | Description |
|----------|-------------|
| [Ability System](./ability-system.md) | Activated abilities, triggers, and ability evaluation |
| [Combat Resolution](./combat-resolution.md) | Combat damage, blocking, and creature destruction |
| [Cost System](./cost-system.md) | Cost validation, payment, and atomic cost handling |
| [Creature Stats](./creature-stats.md) | Power, toughness, and counters system |
| [Effect System](./effect-system.md) | Effect implementation and execution context |
| [Game Events](./game-events.md) | Event types and trigger evaluation |
| [Mana System](./mana-system.md) | Mana pool, mana production and consumption |
| [Stack and Priority](./stack-and-priority.md) | Stack resolution and priority system |
| [Static Abilities](./static-abilities.md) | Consultative keywords (Flying, Reach, Vigilance) |

### Development

| Document | Description |
|----------|-------------|
| [Testing Guide](./testing-guide.md) | Test helpers, patterns, and best practices |

## Project Status

Echomancy is in MVP phase, focusing on fundamental game mechanics:

- Turn system with all phases
- Playing lands and casting spells
- Priority system with stack resolution
- Triggered and activated abilities
- Creature combat with damage resolution
- Creature stats (power, toughness, +1/+1 counters)
- Mana pool (basic operations: add, spend, clear)
- Cost system (mana, tap, sacrifice costs with atomic payment)
- Static abilities (Flying, Reach, Vigilance)
- Game state export (complete, neutral export for UI and external systems)
- Game snapshot (player-relative UI views with visibility filtering)

## Source Code Structure

```
src/echomancy/
├── domainmodel/     # Core game engine (closed, stable)
│   ├── abilities/       # Ability system
│   ├── cards/           # Card definitions and instances
│   ├── costs/           # Cost system
│   │   └── impl/        # Concrete cost types (ManaCost, TapSelfCost, etc.)
│   ├── effects/         # Effect interface and implementations
│   │   └── impl/        # Concrete effect classes
│   ├── game/            # Core game engine
│   │   └── __tests__/   # Test suite
│   ├── targets/         # Targeting system
│   ├── triggers/        # Trigger definitions
│   └── zones/           # Game zones
└── infrastructure/  # UI-facing infrastructure
    └── ui/              # Game snapshot and UI contracts
        └── __tests__/   # Snapshot tests
```
