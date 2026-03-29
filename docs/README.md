# Echomancy Documentation

Echomancy is a Magic: The Gathering game engine built with Domain-Driven Design principles in Rust.

## Documentation Index

### Core Concepts

| Document | Description |
|----------|-------------|
| [Architecture](./architecture.md) | Core design principles and architectural patterns |
| [UI Architecture](./ui-architecture.md) | How the Bevy UI interacts with the game engine |
| [CLIPS Integration](./architecture-clips-integration.md) | CLIPS rules engine design specification |
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

Echomancy has completed its core engine and CLIPS integration:

- Turn system with all phases and state-based actions
- Playing lands, casting spells with target selection
- CLIPS 6.4.2 rules engine for card-specific effects
- Priority system with stack resolution
- Triggered and activated abilities (wired to CLIPS)
- Creature combat with damage resolution
- Mana pool and cost payment with auto-pay
- Static abilities (Flying, Reach, Vigilance, Haste, Flash)
- MTGJSON card data loader
- 820+ tests across the workspace

## Source Code Structure

```
crates/
├── echomancy-core/src/          # Core game engine (pure Rust library, zero Bevy dependency)
│   ├── domain/                  # Domain model
│   │   ├── cards/               # Card definitions, catalog, prebuilt decks
│   │   ├── game/                # Game aggregate root (split by responsibility)
│   │   │   ├── mod.rs           # Game struct, constructors, apply() dispatcher
│   │   │   ├── stack_resolution.rs  # Spell/ability resolution, CLIPS integration
│   │   │   ├── zone_transitions.rs  # Enter battlefield, move to graveyard
│   │   │   ├── sba.rs           # State-based actions
│   │   │   ├── priority.rs      # Priority assignment and passing
│   │   │   └── ...              # Command handlers (cast_spell, play_land, etc.)
│   │   ├── rules_engine.rs      # RulesEngine trait (technology-agnostic)
│   │   ├── services/            # Domain services (combat, mana payment, etc.)
│   │   └── value_objects/       # ManaPool, PermanentState, TurnState, etc.
│   └── infrastructure/          # Infrastructure layer
│       ├── clips/               # CLIPS rules engine (safe FFI wrapper)
│       ├── legal_actions.rs     # Compute what a player can legally do
│       ├── mtgjson.rs           # MTGJSON AtomicCards.json parser
│       └── game_snapshot.rs     # Player-relative game state views
├── echomancy-bevy/src/          # Bevy binary (UI rendering)
│   ├── main.rs                  # Entry point
│   └── plugins/                 # Bevy plugins (Game, UI, HUD, etc.)
├── clips-sys/                   # Raw C FFI bindings to CLIPS 6.4.2
└── rules/                       # CLIPS rule files
    ├── core/templates.clp       # All deftemplates (embedded in binary)
    └── cards/                   # Card-specific .clp rules
```
