# Echomancy Documentation

Echomancy is a Magic: The Gathering game engine built with Domain-Driven Design principles in TypeScript.

## Documentation Index

### Core Concepts

| Document | Description |
|----------|-------------|
| [Architecture](./architecture.md) | Core design principles and architectural patterns |
| [Turn Structure](./turn-structure.md) | Game phases, steps, and turn progression |
| [Zones and Cards](./zones-and-cards.md) | Game zones, card definitions, and instances |

### Systems

| Document | Description |
|----------|-------------|
| [Ability System](./ability-system.md) | Activated abilities, triggers, and ability evaluation |
| [Cost System](./cost-system.md) | Cost validation, payment, and atomic cost handling |
| [Effect System](./effect-system.md) | Effect implementation and execution context |
| [Game Events](./game-events.md) | Event types and trigger evaluation |
| [Mana System](./mana-system.md) | Mana pool, mana production and consumption |
| [Stack and Priority](./stack-and-priority.md) | Stack resolution and priority system |

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
- Creature combat basics
- Mana pool (basic operations: add, spend, clear)
- Cost system (mana, tap, sacrifice costs with atomic payment)

## Source Code Structure

```
src/echomancy/domainmodel/
├── abilities/       # Ability system
├── cards/           # Card definitions and instances
├── costs/           # Cost system
│   └── impl/        # Concrete cost types (ManaCost, TapSelfCost, etc.)
├── effects/         # Effect interface and implementations
│   └── impl/        # Concrete effect classes
├── game/            # Core game engine
│   └── __tests__/   # Test suite
├── targets/         # Targeting system
├── triggers/        # Trigger definitions
└── zones/           # Game zones
```
