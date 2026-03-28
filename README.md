# Echomancy

A Magic: The Gathering game engine built with Domain-Driven Design principles.

## About

Rust implementation of MTG game engine, focusing on clean architecture and robust domain modeling.

**Current Features:**
- Turn system with all phases (Untap, Upkeep, Draw, Main, Combat, Cleanup)
- Playing lands from hand to battlefield
- Casting spells to the stack
- Priority system with stack resolution
- Triggered and activated abilities
- Static abilities (consultative keywords: Flying, Reach, Vigilance)
- Mana pool and cost payment system
- Domain-driven design with comprehensive test coverage

**Tech Stack:**
- Rust (latest stable)
- Bevy 0.18 (game engine / UI)
- Cargo workspace (echomancy-core + echomancy-bevy)
- thiserror (typed errors)
- serde (serialization)
- cargo clippy (linting)
- cargo test (testing)

## Development

```bash
# Build the project
cargo build

# Run the game
cargo run -p echomancy-bevy

# Run tests
cargo test

# Lint
cargo clippy

# Build release
cargo build --release
```

## Documentation

### Core Concepts
- [Architecture](docs/architecture.md) - DDD principles and design philosophy
- [Turn Structure](docs/turn-structure.md) - Phases, steps, and turn progression
- [Zones and Cards](docs/zones-and-cards.md) - Zone system and card model

### Game Systems
- [Ability System](docs/ability-system.md) - Activated and triggered abilities
- [Effect System](docs/effect-system.md) - How effects execute and mutate state
- [Static Abilities](docs/static-abilities.md) - Consultative keywords (Flying, Reach, Vigilance)
- [Mana System](docs/mana-system.md) - Mana pool and cost payment
- [Combat Resolution](docs/combat-resolution.md) - Combat mechanics and damage
- [Stack and Priority](docs/stack-and-priority.md) - The stack and priority system
- [Creature Stats](docs/creature-stats.md) - Power/Toughness calculation

### Technical Reference
- [Game Events](docs/game-events.md) - Event types and trigger system
- [Cost System](docs/cost-system.md) - Cost types and payment
- [Commands and Queries](docs/commands-and-queries.md) - Application layer patterns
- [Game Snapshot](docs/game-snapshot.md) - UI-facing game state views
- [Game State Export](docs/game-state-export.md) - Raw engine state export
- [UI Architecture](docs/ui-architecture.md) - Bevy UI structure
- [Testing Guide](docs/testing-guide.md) - Test helpers and patterns

## Project Management

See [BACKLOG.md](docs/specs/BACKLOG.md) for current status, MVP scope, and active work.
