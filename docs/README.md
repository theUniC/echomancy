# Echomancy Documentation

Welcome to the Echomancy documentation. Echomancy is a Magic: The Gathering game engine built with Domain-Driven Design principles in TypeScript.

## Overview

This documentation consolidates the design principles, architectural decisions, and implementation details that were previously scattered across code comments. Use this as your primary reference when working with the engine.

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
| [Effect System](./effect-system.md) | Effect implementation and execution context |
| [Game Events](./game-events.md) | Event types and trigger evaluation |
| [Stack and Priority](./stack-and-priority.md) | Stack resolution and priority system |

### Development

| Document | Description |
|----------|-------------|
| [Testing Guide](./testing-guide.md) | Test helpers, patterns, and best practices |

## Quick Start

```bash
# Install dependencies
bun install

# Run development server
bun dev

# Run tests
bun test

# Lint & format
bun run lint && bun run format
```

## Project Status

Echomancy is in MVP phase, focusing on fundamental game mechanics:

- Turn system with all phases
- Playing lands and casting spells
- Priority system with stack resolution
- Triggered and activated abilities
- Creature combat basics

See [Architecture](./architecture.md) for current limitations and planned features.

## Tech Stack

- **Runtime**: Bun
- **Framework**: Next.js 16 with App Router
- **UI**: React 19 with React Compiler
- **Language**: TypeScript (strict mode)
- **Styling**: Tailwind CSS v4
- **Testing**: Vitest
- **Linting**: Biome

## Source Code Structure

```
src/echomancy/domainmodel/
├── abilities/       # Ability system (ActivatedAbility, type guards)
├── cards/           # Card definitions and instances
├── effects/         # Effect interface and implementations
│   └── impl/        # Concrete effect classes
├── game/            # Core game engine
│   └── __tests__/   # Comprehensive test suite
├── targets/         # Targeting system (future)
├── triggers/        # Trigger definitions
└── zones/           # Game zones
```
