# Commands and Queries

CQRS-lite pattern for application-level operations in Echomancy.

## Key Concepts

- **Command**: Immutable data class representing an intention to change state
- **CommandHandler**: Validates and executes the command
- **Commands are fire-and-forget**: No return value (void)
- **Queries** (future): Read operations that return data without mutating state
- **Application boundary**: Commands cross the boundary between API and domain

**For REST API design**: See `api-conventions.md`

## How It Works

Each command lives in its own folder with a single file containing both Command and CommandHandler classes:

```
src/echomancy/application/command/
├── create-game/
│   ├── CreateGameCommand.ts
│   └── CreateGameCommand.test.ts
└── join-game/
    ├── JoinGameCommand.ts
    └── JoinGameCommand.test.ts
```

### Command Class

Simple data transfer object:

```typescript
export class CreateGameCommand {
  constructor(public id: string) {}
}
```

### CommandHandler Class

Validation + execution:

```typescript
export class CreateGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle(message: CreateGameCommand) {
    // Validate, execute, persist
  }
}
```

## Rules

### Command Structure
- Use `public` constructor parameters
- All parameters are primitives or value objects
- Immutable after construction
- No methods, no logic

### Naming Convention
- Command: `{Action}{Entity}Command` (e.g., `CreateGameCommand`)
- Handler: `{Action}{Entity}CommandHandler`
- File: `{Action}{Entity}Command.ts`
- Folder: `{action}-{entity}/` (kebab-case)

### Validation Order

Handlers validate in this order:

1. **Input validation**: Check inputs are well-formed
2. **Existence checks**: Verify required entities exist
3. **Domain validation**: Let domain model enforce its rules

Example:
```typescript
handle(message: JoinGameCommand) {
  // 1. Input validation
  if (!isValidUUID(message.gameId)) {
    throw new InvalidGameIdError(message.gameId)
  }

  // 2. Existence check
  const game = this.gameRepository.byId(message.gameId)
  if (!game) throw new GameNotFoundError(message.gameId)

  // 3. Domain validation (throws if invalid)
  game.addPlayer(new Player(message.playerId, message.playerName))
}
```

### Error Handling

Commands throw domain-specific errors:

| Error | When |
|-------|------|
| `InvalidGameIdError` | Game ID not valid UUID |
| `InvalidPlayerIdError` | Player ID not valid UUID |
| `GameNotFoundError` | Game doesn't exist |
| `DuplicatePlayerError` | Player already in game |

**Implementation**: All errors extend `GameError` in `src/echomancy/domainmodel/game/errors/`

### Testing

Each command has comprehensive tests:

```typescript
describe("JoinGameCommand", () => {
  it("throws when game ID is invalid", () => {
    // Test input validation
  })

  it("adds player to existing game", () => {
    // Test happy path
  })
})
```

**Full examples**: See `*Command.test.ts` files in `src/echomancy/application/command/`
