# Game Setup: Deck Loading and Opening Hand

## Overview

Replace the current hardcoded starting hands with a real game setup flow: load decks → shuffle → draw 7 cards. This establishes the foundation for a real TCG game start where each player begins with a randomized hand from their deck.

### Design Goals
- Eliminate the temporary bootstrap that hardcodes specific cards in hands
- Load deck lists from predefined card pools
- Shuffle library zones randomly
- Draw 7-card opening hands using existing draw logic
- Maintain backward compatibility with existing UI phases

### Relationship to Other Systems
- **Zones**: Uses existing HAND and LIBRARY zones
- **Card Instances**: Creates instances from deck definitions
- **Zone Transitions**: Uses existing `moveCard()` and draw logic
- **Game State Export**: Library zone already exports card counts and other data

## User Stories

**As a player**, I want to start a game with a shuffled deck and random opening hand, so that each game feels unique and strategic decisions start from turn 1.

**As a developer**, I want reproducible deck configurations for testing, so that I can verify game logic without manually creating card instances.

## Player Experience

### Before This Feature
1. Player opens the game UI
2. Both players have identical, hardcoded hands with specific test cards
3. No library zone exists - cards are magically available
4. No strategic variation between games

### After This Feature
1. Player opens the game UI
2. Game loads a predefined deck list for each player
3. Deck is shuffled randomly in the library zone
4. Each player draws 7 cards from their library to form their opening hand
5. The game begins with randomized hands and full libraries

### What the Player Sees
- **Library count**: Shows 53 cards remaining after initial draw (60 - 7)
- **Hand**: 7 random cards from their deck
- **No visible difference**: The UI remains unchanged - this is a backend change

## Game Rules & Mechanics

### MTG Game Setup Rules

Per MTG Comprehensive Rules:

**103.2** Each player's deck must contain at least 60 cards for Constructed formats. For the MVP, we use 60-card decks.

**103.3** Each player begins the game with 20 life in two-player formats.

**103.4** Each player draws seven cards from their library to form their opening hand.

**103.5** Players may mulligan their hands (deferred - see Out of Scope).

### Echomancy Implementation

**Deck List Format**: For the MVP, decks are represented as arrays of `CardDefinition` objects. The same card can appear multiple times (e.g., 4x Forest, 4x Llanowar Elves).

**Card Instance Creation**: When a deck is loaded, each `CardDefinition` becomes a unique `CardInstance` with:
- A unique `instanceId` (generated using `crypto.randomUUID()`)
- A reference to its `CardDefinition` via `definitionId`
- An `ownerId` matching the player who owns this deck

**Shuffle**: Randomize the order of cards in the library zone using a standard array shuffle (Fisher-Yates or similar).

**Draw Opening Hand**: Use the existing draw logic (if it exists) or implement a simple "move top 7 cards from LIBRARY to HAND" operation.

### Edge Cases

**Empty library during initial draw**: Should never happen in MVP since decks are 60 cards and we only draw 7. No special handling needed.

**Deck lists with < 60 cards**: For MVP, assume deck lists are valid. Validation can be added later.

**Duplicate card instance IDs**: Use `crypto.randomUUID()` which has negligible collision probability.

**Deterministic shuffling for tests**: Test helpers should accept an optional seed or use a seeded random number generator to ensure reproducible shuffle order.

## Acceptance Criteria

### Backend (Engine)

**Deck Loading**
- [ ] `Game` constructor accepts deck lists for each player
- [ ] Each card in the deck list creates a unique `CardInstance`
- [ ] All instances are placed in the player's LIBRARY zone
- [ ] Each instance has a unique `instanceId`

**Shuffle**
- [ ] Library zone is shuffled randomly after deck loading
- [ ] Shuffle order is different between games (not deterministic in production)
- [ ] Test helper exists for deterministic shuffle (seeded RNG)

**Opening Hand Draw**
- [ ] After shuffle, each player draws 7 cards from their library
- [ ] Cards move from LIBRARY to HAND (top 7 cards)
- [ ] Library count decreases by 7 after draw

**Game State Export**
- [ ] Library zone exports card count (should show 53 after opening draw)
- [ ] Hand zone shows 7 cards
- [ ] Cards in hand are properly exported with all metadata

**Tests**
- [ ] Test: Deck loading creates unique instances
- [ ] Test: Shuffle randomizes order (compare two shuffles, expect different orders)
- [ ] Test: Opening hand draw moves 7 cards from library to hand
- [ ] Test: Game state exports correct library count after setup

### UI (Frontend)

**No UI changes required** - This is a backend-only feature. The existing UI already displays library counts and hand contents. Once the backend populates these zones correctly, the UI will reflect it automatically.

**Manual QA Verification**
- [ ] Load game in browser
- [ ] Verify library shows 53 cards (not 60)
- [ ] Verify hand shows 7 cards
- [ ] Refresh and verify hand contents are different (shuffle worked)
- [ ] Verify cards in hand match card definitions from deck list

## Out of Scope

### Explicitly NOT Included
- **Mulligan system**: Players cannot mulligan their opening hands. They keep the first 7 cards drawn.
- **Deck builder UI**: Deck lists are hardcoded in the backend for MVP. No UI for deck construction.
- **Deck validation**: No validation of deck legality (card limits, format rules, etc.)
- **Custom deck selection**: Both players use predefined deck lists. No deck selection UI.
- **London mulligan / Vancouver mulligan**: All mulligan rules deferred.

### Future Considerations
- Add mulligan system (London mulligan with scry 1)
- Add deck validation (60-card minimum, 4-of limit, format legality)
- Add deck builder UI for custom deck creation
- Add deck selection UI (choose from saved decks)
- Add deck import/export (text list, MTGO format, etc.)

## Technical Notes (for Implementation Team)

### Where to Implement

**Engine (Backend)**:
- `src/echomancy/domainmodel/game/Game.ts`: Update constructor to accept deck lists, call shuffle and draw logic
- `src/echomancy/domainmodel/game/DeckLoader.ts` (new): Helper for creating card instances from deck definitions
- `src/echomancy/domainmodel/game/__tests__/Game.setup.test.ts` (new): Tests for deck loading, shuffle, and draw

**Application Layer**:
- `src/echomancy/application/commands/StartGameCommandHandler.ts`: Pass deck lists to `Game` constructor when creating a new game

**API Layer**:
- `src/app/api/games/route.ts`: Accept deck list IDs or use default deck lists when creating a game

### Suggested Test Deck Lists

For MVP, use simple, functional deck lists:

**Red Burn Deck** (Player 1):
- 20x Mountain
- 4x Lightning Bolt
- 4x Shock
- 4x Lava Spike
- (... fill to 60 cards with more burn spells and creatures)

**Green Ramp Deck** (Player 2):
- 24x Forest
- 4x Llanowar Elves
- 4x Llanowar Elves
- (... fill to 60 cards with ramp spells and big creatures)

### Dependencies

**Required Before This Feature**:
- ✅ LIBRARY zone exists
- ✅ HAND zone exists
- ✅ `moveCard()` or draw logic exists
- ✅ `CardDefinition` and `CardInstance` types exist

**Blocks Future Work**:
- Mulligan system
- Deck builder
- Real two-player games
