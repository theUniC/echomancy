# API Conventions

RESTful API design conventions for Echomancy HTTP endpoints.

## Key Concepts

- **Resource-oriented**: URLs represent resources (games, players), not actions
- **Standard HTTP**: POST (create), GET (read), PUT (replace), PATCH (update), DELETE (remove)
- **JSON**: Request and response bodies use `application/json`
- **Nested resources**: Sub-resources under parent (e.g., `/games/:id/players`)

**For command/query implementation patterns**: See `commands-and-queries.md`

## How It Works

### URL Structure

Resources use plural nouns, nested where appropriate:
```
/api/games                    # Game collection
/api/games/:gameId            # Single game
/api/games/:gameId/players    # Players in a game
/api/games/:gameId/actions    # Actions applied to a game
/api/games/:gameId/state      # Game state snapshot
```

### HTTP Methods

| Method | Purpose | Example |
|--------|---------|---------|
| POST | Create | `POST /api/games` |
| GET | Read | `GET /api/games/:id` |
| PUT | Replace | `PUT /api/games/:id` |
| PATCH | Update | `PATCH /api/games/:id` |
| DELETE | Remove | `DELETE /api/games/:id` |

### Request/Response Format

**Request**: JSON body
```
POST /api/games
{ "gameId": "uuid" }
```

**Success**: Data + metadata
```json
{ "data": {...}, "meta": {"timestamp": "..."} }
```

**Error**: Code + message
```json
{ "error": {"code": "GAME_NOT_FOUND", "message": "..."} }
```

## Rules

### Status Codes

| Code | Use Case |
|------|----------|
| 200 OK | Successful read/update |
| 201 Created | Successful creation |
| 204 No Content | Successful deletion |
| 400 Bad Request | Malformed input |
| 404 Not Found | Resource missing |
| 409 Conflict | Business rule violation |
| 422 Unprocessable | Domain-invalid action |
| 500 Server Error | Unexpected error |

### Route Structure

```
src/app/api/games/
├── route.ts                    # POST /api/games
└── [gameId]/
    ├── route.ts                # GET /api/games/:id
    ├── players/route.ts        # POST, GET players
    ├── actions/route.ts        # POST actions
    └── state/route.ts          # GET state
```

### Handler Pattern

Each route handler:
1. Parse request
2. Validate format
3. Call Command/Query (see `commands-and-queries.md`)
4. Map errors to status codes
5. Return JSON

