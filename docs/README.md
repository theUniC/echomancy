# Echomancy Documentation

Echomancy is a Magic: The Gathering game engine built with Domain-Driven Design principles in Rust.

## Documentation Index

### Core Concepts

| Document | Description |
|----------|-------------|
| [Architecture](./architecture.md) | Core design principles and architectural patterns |
| [UI Architecture](./ui-architecture.md) | How the Bevy UI interacts with the game engine |
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

### Specifications

Feature specifications and design documents are organized in `docs/specs/`:

```
docs/specs/
├── active/     # Currently being implemented (1-2 specs max)
├── backlog/    # Prioritized specs waiting to be implemented
├── done/       # Completed specifications (reference)
└── features/   # Feature design documents and architecture decisions
```

See `AGENTS.md` for the complete specification workflow.

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
crates/
├── echomancy-core/src/          # Core game engine (pure Rust library, zero Bevy dependency)
│   ├── domain/                  # Domain model
│   │   ├── abilities.rs         # Ability system
│   │   ├── cards/               # Card definitions and instances
│   │   ├── costs.rs             # Cost system
│   │   ├── effects.rs           # Effect interface and implementations
│   │   ├── entities/            # Zone entities (Battlefield, Hand, etc.)
│   │   ├── game/                # Core game engine (aggregate root)
│   │   ├── services/            # Domain services
│   │   ├── specifications/      # Business rule specifications
│   │   ├── targets.rs           # Targeting system
│   │   ├── triggers.rs          # Trigger definitions
│   │   └── value_objects/       # Value objects (ManaPool, PermanentState, etc.)
│   ├── application/             # Application layer (commands/queries)
│   └── infrastructure/          # Infrastructure (UI contracts, repositories)
└── echomancy-bevy/src/          # Bevy binary (UI rendering)
    ├── main.rs                  # Entry point
    └── plugins/                 # Bevy plugins (Game, UI, HUD, etc.)
```
