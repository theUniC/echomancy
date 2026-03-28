# Commands and Queries

CQRS-lite pattern for application-level operations in Echomancy.

## Key Concepts

- **Command**: Data structure representing an intention to change state
- **CommandHandler**: Validates and executes the command
- **Commands are fire-and-forget**: No return value
- **Queries** (future): Read operations that return data without mutating state
- **Application boundary**: Commands cross the boundary between UI and domain

## How It Works

Commands and handlers live in the application layer:

```
crates/echomancy-core/src/application/
├── mod.rs
└── commands/
    ├── create_game.rs
    └── join_game.rs
```

### Command Pattern

Commands are simple data transfer structs:

```rust
pub(crate) struct CreateGameCommand {
    pub id: String,
}
```

### CommandHandler

Validation + execution:

```rust
pub(crate) struct CreateGameCommandHandler<'a> {
    game_repository: &'a dyn GameRepository,
}

impl<'a> CreateGameCommandHandler<'a> {
    pub fn handle(&self, command: CreateGameCommand) -> Result<(), GameError> {
        // Validate, execute, persist
    }
}
```

## Rules

### Command Structure
- All parameters are primitives or value objects
- Immutable after construction
- No methods, no logic

### Naming Convention
- Command: `{Action}{Entity}Command` (e.g., `CreateGameCommand`)
- Handler: `{Action}{Entity}CommandHandler`
- Module: `{action}_{entity}.rs` (snake_case)

### Validation Order

Handlers validate in this order:

1. **Input validation**: Check inputs are well-formed
2. **Existence checks**: Verify required entities exist
3. **Domain validation**: Let domain model enforce its rules

### Error Handling

Commands return `Result<(), GameError>` with typed errors:

| Error | When |
|-------|------|
| `InvalidGameIdError` | Game ID not valid |
| `InvalidPlayerIdError` | Player ID not valid |
| `GameNotFoundError` | Game doesn't exist |
| `DuplicatePlayerError` | Player already in game |

**Implementation**: All errors defined in `crates/echomancy-core/src/domain/errors.rs`

### Testing

Each command has comprehensive tests in inline `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throws_when_game_id_is_invalid() {
        // Test input validation
    }

    #[test]
    fn adds_player_to_existing_game() {
        // Test happy path
    }
}
```

**Full examples**: See command modules in `crates/echomancy-core/src/application/`
