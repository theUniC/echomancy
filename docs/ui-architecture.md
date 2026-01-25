# UI Architecture

How the React UI layer interacts with the game engine through HTTP boundaries.

## Key Concepts

- **Network Boundary** - Frontend and backend are separated by HTTP
- **No Direct Imports** - React cannot import server-side code
- **API Routes** - Next.js routes handle HTTP and call application layer
- **Data Contracts** - GameSnapshot (player-relative) and GameStateExport (complete state)

## How It Works

**Layer Structure**:
```
src/app/                         # Next.js (client + API routes)
├── (client components)          # React UI - HTTP requests only
└── api/                         # API routes - calls application layer

src/echomancy/application/       # Commands/Queries (server-only)
    ↓
src/echomancy/domainmodel/       # Game engine (server-only)
    ↓
src/echomancy/infrastructure/    # Repositories (server-only)
```

**Communication Flow**: React → fetch() → API route → CommandHandler/QueryHandler → Game → Repository

**Import Rules**:
- React: No server imports. Use fetch() to call API routes.
- API routes: Can import application/ and infrastructure/
- Commands/Queries: Can import domainmodel/ and infrastructure/
- Domain model: Can import infrastructure/ interfaces only

See `docs/api-conventions.md` for REST API design and `docs/commands-and-queries.md` for application layer patterns.

## Rules

- Client and server communicate only via HTTP/JSON
- Client receives plain JSON objects, never class instances
- API routes are the boundary (parse HTTP, call handlers, return JSON)
- GameSnapshot is filtered and player-relative
- GameStateExport is complete and unfiltered
