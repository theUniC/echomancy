# Commands and Queries

This document describes the CQRS-lite pattern used in Echomancy for application-level operations.

## Overview

Echomancy uses a simplified Command pattern for write operations. Commands represent intentions to change the system state, while Queries (when implemented) will represent read operations.

```
src/echomancy/application/
├── command/
│   ├── create-game/
│   │   ├── CreateGameCommand.ts
│   │   └── CreateGameCommand.test.ts
│   └── join-game/
│       ├── JoinGameCommand.ts
│       └── JoinGameCommand.test.ts
└── query/
    └── (future queries go here)
```

## Command Pattern

### Structure

Each command consists of two classes in a single file:

1. **Command**: A simple data class holding the operation's parameters
2. **CommandHandler**: Executes the command with validation and domain logic

### Naming Convention

- Command: `{Action}{Entity}Command` (e.g., `CreateGameCommand`, `JoinGameCommand`)
- Handler: `{Action}{Entity}CommandHandler`
- File: `{Action}{Entity}Command.ts`
- Test: `{Action}{Entity}Command.test.ts`
- Folder: `{action}-{entity}/` (kebab-case)

### Command Class

Commands are immutable data transfer objects:

```typescript
export class CreateGameCommand {
  constructor(public id: string) {}
}

export class JoinGameCommand {
  constructor(
    public gameId: string,
    public playerId: string,
    public playerName: string,
  ) {}
}
```

**Rules:**
- Use `public` constructor parameters for simplicity
- All parameters should be primitives or simple value objects
- Commands are immutable after construction
- No methods, no logic - just data

### CommandHandler Class

Handlers contain validation and execution logic:

```typescript
export class CreateGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle(message: CreateGameCommand) {
    // 1. Validate input
    if (!isValidUUID(message.id)) {
      throw new InvalidGameIdError(message.id)
    }

    // 2. Execute domain logic
    const game = Game.create(message.id)

    // 3. Persist changes
    this.gameRepository.add(game)
  }
}
```

**Rules:**
- Inject dependencies via constructor (repositories, services)
- Method is always named `handle()`
- Validate inputs before executing domain logic
- Throw domain-specific errors for invalid operations
- No return value for commands (void) - commands are fire-and-forget

### Validation Order

1. **Input validation**: Check that inputs are well-formed (valid UUIDs, non-empty strings)
2. **Existence checks**: Verify required entities exist
3. **Domain validation**: Let the domain model enforce its own rules

```typescript
handle(message: JoinGameCommand) {
  // 1. Input validation
  if (!isValidUUID(message.gameId)) {
    throw new InvalidGameIdError(message.gameId)
  }
  if (!isValidUUID(message.playerId)) {
    throw new InvalidPlayerIdError(message.playerId)
  }

  // 2. Existence check
  const game = this.gameRepository.byId(message.gameId)
  if (!game) {
    throw new GameNotFoundError(message.gameId)
  }

  // 3. Domain validation (handled by Game.addPlayer)
  const player = new Player(message.playerId, message.playerName)
  game.addPlayer(player)  // Throws DuplicatePlayerError, CannotAddPlayerAfterStartError
}
```

## Error Handling

Commands throw domain-specific errors:

| Error | When |
|-------|------|
| `InvalidGameIdError` | Game ID is not a valid UUID |
| `InvalidPlayerIdError` | Player ID is not a valid UUID |
| `GameNotFoundError` | Game does not exist |
| `DuplicatePlayerError` | Player already in game |
| `CannotAddPlayerAfterStartError` | Game already started |

All errors extend `GameError` and provide descriptive messages.

## Testing Commands

Each command has a corresponding test file covering:

1. **Validation errors**: Invalid inputs should throw appropriate errors
2. **Existence errors**: Missing entities should throw not-found errors
3. **Happy path**: Valid inputs should succeed
4. **Domain errors**: Domain rules should be enforced

```typescript
describe("JoinGameCommand", () => {
  it("throws when game ID is invalid", () => {
    const handler = new JoinGameCommandHandler(new InMemoryGameRepository())

    expect(() => {
      handler.handle(new JoinGameCommand("invalid", uuidv4(), "Player"))
    }).toThrow(InvalidGameIdError)
  })

  it("adds player to existing game", () => {
    const repo = new InMemoryGameRepository()
    const gameId = uuidv4()
    repo.add(Game.create(gameId))

    const handler = new JoinGameCommandHandler(repo)
    handler.handle(new JoinGameCommand(gameId, uuidv4(), "Player 1"))

    expect(repo.byId(gameId)?.getPlayersInTurnOrder()).toHaveLength(1)
  })
})
```

## Queries (Future)

Queries will follow a similar pattern for read operations:

```
src/echomancy/application/query/
└── get-game-state/
    ├── GetGameStateQuery.ts
    └── GetGameStateQuery.test.ts
```

Unlike commands:
- Queries return data
- Queries are idempotent
- Queries don't modify state

## When to Create a Command

Create a command when:
- An operation changes system state
- The operation crosses the application boundary (API call, user action)
- You need to validate inputs before touching the domain

Don't create a command when:
- The operation is internal to the domain (use domain methods directly)
- You're just reading data (use a query or repository directly)

## Existing Commands

| Command | Purpose |
|---------|---------|
| `CreateGameCommand` | Creates a new game with a given ID |
| `JoinGameCommand` | Adds a player to an existing game |

## Adding a New Command

1. Create folder: `src/echomancy/application/command/{action}-{entity}/`
2. Create command file with Command + CommandHandler classes
3. Create test file with full coverage
4. Run `bun test && bun run lint && bun run format`
5. Commit with descriptive message
