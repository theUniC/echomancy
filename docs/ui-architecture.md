# UI Architecture

How the Bevy UI layer interacts with the game engine through the ECS boundary.

## Key Concepts

- **ECS Boundary** - Game engine lives as a Bevy Resource, UI reads derived state
- **No Direct Imports** - Bevy systems read from Resources/Components, not Game directly
- **Event-Driven Actions** - Player actions become Bevy Events applied to the Game resource
- **Data Contracts** - GameSnapshot (player-relative) and GameStateExport (complete state)

## How It Works

**Layer Structure**:
```
crates/echomancy-bevy/src/       # Bevy binary (UI rendering)
├── main.rs                      # Entry point
└── plugins/                     # Bevy plugins
    ├── game_plugin              # GameState resource, action handling
    ├── ui_plugin                # Camera, root layout
    ├── hud_plugin               # Turn info, life totals, buttons
    ├── battlefield_plugin       # Card rendering on battlefield
    ├── hand_plugin              # Hand display, card interactions
    └── error_plugin             # Error message display

crates/echomancy-core/src/       # Pure Rust library (no Bevy dependency)
├── domain/                      # Game engine (domain model)
├── application/                 # Commands/Queries
└── infrastructure/              # UI contracts (GameSnapshot, etc.)
```

**Communication Flow**: Bevy Input System -> Bevy Event -> Action Handler System -> game.apply() -> GameState Resource mutated -> Snapshot refresh system -> UI systems read snapshot

**Data Flow**:
- Game state wrapped as `Resource<GameState>`
- `GameSnapshot` derived when `GameState` changes
- UI systems read the snapshot resource to render
- Player actions dispatched as Bevy Events

See `docs/commands-and-queries.md` for application layer patterns.

## Rules

- echomancy-core has zero Bevy dependency (pure Rust library)
- Bevy systems never call domain methods directly on Game — use the action event pattern
- GameSnapshot is filtered and player-relative
- GameStateExport is complete and unfiltered
- Plugin architecture: one EchomancyPlugin composed of sub-plugins
- Game aggregate lives as a Bevy Resource
- Player actions become Bevy Events
- Use States enum for game phases, StateScoped for cleanup
