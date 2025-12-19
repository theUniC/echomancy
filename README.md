# Echomancy

A Magic: The Gathering game engine built with Domain-Driven Design principles.

## About

Echomancy is a TypeScript implementation of a Magic: The Gathering game engine, focusing on clean architecture and robust domain modeling.

**Current Features:**
- Turn system with all phases (Untap, Upkeep, Draw, Main, Combat, Cleanup)
- Playing lands from hand to battlefield
- Casting spells to the stack
- Priority system with stack resolution
- Domain-driven design with comprehensive test coverage

**Tech Stack:**
- Next.js 16 with App Router
- React 19 with React Compiler
- TypeScript (strict mode)
- Tailwind CSS v4
- Biome (linting & formatting)
- Vitest (testing)
- Bun runtime

## Development

```bash
# Install dependencies
bun install

# Run development server
bun dev

# Run tests
bun test

# Lint & format
bun run lint
bun run format
```

## Project Status

This is an early-stage project implementing core Magic: The Gathering rules incrementally. Currently in MVP phase focusing on fundamental game mechanics.
