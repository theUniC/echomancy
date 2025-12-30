# API Conventions

This document defines RESTful API conventions for Echomancy.

## Resource Naming

Use plural nouns for resources. Nest sub-resources under their parent.

```
/api/games                    # Game collection
/api/games/:gameId            # Single game
/api/games/:gameId/players    # Players in a game
/api/games/:gameId/actions    # Actions applied to a game
/api/games/:gameId/state      # Game state (read-only)
```

## HTTP Methods

| Method | Purpose | Example |
|--------|---------|---------|
| `POST` | Create resource | `POST /api/games` |
| `GET` | Read resource | `GET /api/games/:id` |
| `PUT` | Replace resource | `PUT /api/games/:id` |
| `PATCH` | Partial update | `PATCH /api/games/:id` |
| `DELETE` | Delete resource | `DELETE /api/games/:id` |

## Common Endpoints

### Games

```
POST   /api/games              # Create a new game
GET    /api/games/:id          # Get game details
DELETE /api/games/:id          # Delete a game
```

### Players

```
POST   /api/games/:id/players  # Join a game (add player)
GET    /api/games/:id/players  # List players in game
DELETE /api/games/:id/players/:playerId  # Leave game
```

### Game Actions

```
POST   /api/games/:id/actions  # Apply an action to the game
GET    /api/games/:id/actions  # Get action history (if needed)
```

### Game State

```
GET    /api/games/:id/state              # Get full game state
GET    /api/games/:id/state?player=p1    # Get player-relative snapshot
```

## Request Format

### Creating a Game

```http
POST /api/games
Content-Type: application/json

{
  "gameId": "uuid-here"
}
```

### Joining a Game

```http
POST /api/games/abc-123/players
Content-Type: application/json

{
  "playerId": "uuid-here",
  "playerName": "Alice"
}
```

### Applying an Action

```http
POST /api/games/abc-123/actions
Content-Type: application/json

{
  "type": "PLAY_LAND",
  "playerId": "p1",
  "cardId": "forest-1"
}
```

## Response Format

### Success Response

```json
{
  "data": { ... },
  "meta": {
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "GAME_NOT_FOUND",
    "message": "Game with ID abc-123 not found"
  }
}
```

## HTTP Status Codes

| Code | When to use |
|------|-------------|
| `200 OK` | Successful GET, PUT, PATCH |
| `201 Created` | Successful POST that creates resource |
| `204 No Content` | Successful DELETE |
| `400 Bad Request` | Invalid input (malformed JSON, missing fields) |
| `404 Not Found` | Resource doesn't exist |
| `409 Conflict` | Business rule violation (duplicate player, invalid action) |
| `422 Unprocessable Entity` | Valid JSON but invalid for domain (illegal game action) |
| `500 Internal Server Error` | Unexpected server error |

## Route Handler Structure

Each route handler in `src/app/api/` should:

1. Parse request body/params
2. Validate input format
3. Call appropriate Command/Query
4. Handle errors and map to HTTP status
5. Return JSON response

### Example: Create Game

```typescript
// src/app/api/games/route.ts
import { NextResponse } from "next/server"
import { CreateGameCommand, CreateGameCommandHandler } from "@/echomancy/application/command/create-game/CreateGameCommand"
import { gameRepository } from "@/lib/repositories"

export async function POST(request: Request) {
  try {
    const body = await request.json()

    if (!body.gameId) {
      return NextResponse.json(
        { error: { code: "MISSING_GAME_ID", message: "gameId is required" } },
        { status: 400 }
      )
    }

    const handler = new CreateGameCommandHandler(gameRepository)
    handler.handle(new CreateGameCommand(body.gameId))

    return NextResponse.json(
      { data: { gameId: body.gameId } },
      { status: 201 }
    )
  } catch (error) {
    if (error instanceof InvalidGameIdError) {
      return NextResponse.json(
        { error: { code: "INVALID_GAME_ID", message: error.message } },
        { status: 400 }
      )
    }
    throw error
  }
}
```

### Example: Apply Action

```typescript
// src/app/api/games/[gameId]/actions/route.ts
import { NextResponse } from "next/server"

export async function POST(
  request: Request,
  { params }: { params: { gameId: string } }
) {
  try {
    const action = await request.json()
    const { gameId } = params

    const game = gameRepository.byId(gameId)
    if (!game) {
      return NextResponse.json(
        { error: { code: "GAME_NOT_FOUND", message: `Game ${gameId} not found` } },
        { status: 404 }
      )
    }

    game.apply(action)

    return NextResponse.json({
      data: { state: game.exportState() }
    })
  } catch (error) {
    if (error instanceof GameError) {
      return NextResponse.json(
        { error: { code: error.code, message: error.message } },
        { status: 422 }
      )
    }
    throw error
  }
}
```

### Example: Get Game State

```typescript
// src/app/api/games/[gameId]/state/route.ts
import { NextResponse } from "next/server"
import { createGameSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"

export async function GET(
  request: Request,
  { params }: { params: { gameId: string } }
) {
  const { gameId } = params
  const { searchParams } = new URL(request.url)
  const playerId = searchParams.get("player")

  const game = gameRepository.byId(gameId)
  if (!game) {
    return NextResponse.json(
      { error: { code: "GAME_NOT_FOUND", message: `Game ${gameId} not found` } },
      { status: 404 }
    )
  }

  const state = game.exportState()

  // If player specified, return player-relative snapshot
  if (playerId) {
    const snapshot = createGameSnapshot(state, playerId, cardRegistry)
    return NextResponse.json({ data: snapshot })
  }

  // Otherwise return full state (for debug)
  return NextResponse.json({ data: state })
}
```

## File Structure

```
src/app/api/
├── games/
│   ├── route.ts                    # POST /api/games
│   └── [gameId]/
│       ├── route.ts                # GET /api/games/:id
│       ├── players/
│       │   └── route.ts            # POST, GET /api/games/:id/players
│       ├── actions/
│       │   └── route.ts            # POST /api/games/:id/actions
│       └── state/
│           └── route.ts            # GET /api/games/:id/state
```
