# API Conventions

> **Note**: This document was written for the original TypeScript/Next.js REST API architecture. The project has since migrated to Rust/Bevy, where the game engine runs as a native binary without HTTP endpoints. The Bevy UI communicates with the game engine directly through Bevy Resources and Events, not REST APIs.
>
> This document is retained for historical reference. For the current architecture, see [UI Architecture](./ui-architecture.md).

## Current Architecture (Rust/Bevy)

In the Rust/Bevy architecture:
- There is no HTTP API layer
- The game engine (`echomancy-core`) is a pure Rust library
- The UI (`echomancy-bevy`) accesses the engine directly as a Bevy Resource
- Player actions are Bevy Events that systems translate into `game.apply()` calls
- Game state is exposed via `GameSnapshot` and `GameStateExport` structs

See `docs/commands-and-queries.md` for application layer patterns and `docs/ui-architecture.md` for the Bevy integration architecture.
