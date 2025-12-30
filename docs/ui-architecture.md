# UI Architecture

This document defines how the UI layer interacts with the game engine.

## Fundamental Rule

**Frontend and Backend are separated by network.**

The React client cannot directly import server-side code. All communication happens via HTTP.

## Layer Structure

```
src/app/                         # Next.js (client components + API routes)
├── (client components)          # React UI - makes HTTP requests
└── api/                         # API routes - handles HTTP, calls application layer

src/echomancy/application/       # Commands and Queries (server-side only)
    ↓ uses
src/echomancy/domainmodel/       # Game engine (server-side only)
    ↓ persisted via
src/echomancy/infrastructure/    # Repositories, persistence
```

## Communication Flow

```
React Client (browser)
    ↓ HTTP POST/GET
Next.js API Routes (server)
    ↓ instantiates and calls
CommandHandler / QueryHandler
    ↓ uses
Game (domainmodel)
    ↓ persisted via
GameRepository (infrastructure)
```

## What Each Layer Can Import

| Layer | Can Import From |
|-------|-----------------|
| React components | Nothing server-side. Uses `fetch()` to API routes |
| API routes | `application/`, `infrastructure/` |
| Commands/Queries | `domainmodel/`, `infrastructure/` |
| Domain model | Only itself and `infrastructure/` interfaces |

## Correct vs Incorrect

```
CORRECT:
  React → fetch("/api/game/create") → API route → CreateGameCommand → Game

INCORRECT:
  React → import { Game } from "domainmodel"     (IMPOSSIBLE - server code)
  React → import { CreateGameCommand } from "application"  (IMPOSSIBLE)
```

## API Routes Responsibility

API routes in `src/app/api/` are the boundary between client and server:

1. Parse and validate HTTP request
2. Instantiate CommandHandler/QueryHandler with dependencies
3. Call handler
4. Return HTTP response (JSON)

```typescript
// src/app/api/game/create/route.ts
export async function POST(request: Request) {
  const body = await request.json()

  const handler = new CreateGameCommandHandler(gameRepository)
  handler.handle(new CreateGameCommand(body.gameId))

  return Response.json({ success: true })
}
```

## Data Contracts

The UI receives data through defined contracts:

- `GameSnapshot` - Player-relative view with visibility filtering
- `GameStateExport` - Complete game state export

These are serialized to JSON and sent over HTTP. The client receives plain JSON objects, not class instances.

## Example: Playing a Land

### Client Side (React)

```typescript
async function playLand(cardId: string) {
  const response = await fetch("/api/game/action", {
    method: "POST",
    body: JSON.stringify({
      type: "PLAY_LAND",
      playerId: currentPlayer,
      cardId
    })
  })
  const newState = await response.json()
  setGameState(newState)
}
```

### Server Side (API Route)

```typescript
// src/app/api/game/action/route.ts
export async function POST(request: Request) {
  const action = await request.json()

  const handler = new ApplyActionCommandHandler(gameRepository)
  handler.handle(new ApplyActionCommand(gameId, action))

  const game = gameRepository.byId(gameId)
  return Response.json(game.exportState())
}
```

## Adding New Features

When adding UI features that interact with the game:

1. Define the API contract (request/response JSON shape)
2. Create API route in `src/app/api/`
3. Create or use existing Command/Query in `application/`
4. Build React component that calls the API
