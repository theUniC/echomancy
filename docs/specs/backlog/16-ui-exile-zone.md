# UI: Exile Zone

## Overview

Display the exile zone for both players.

## User Stories

**As a player**, I want to:
- See cards that have been exiled
- Know how many cards are in exile

## Player Experience

- Exile zone indicator visible for each player
- Shows count and/or viewable list of exiled cards
- Distinct from graveyard (different zone)

## Game Rules & Mechanics

- Exile is a separate zone from graveyard
- Exiled cards are typically public information (face-up)
- Some effects exile face-down (hidden information)

## Acceptance Criteria

- [ ] Exile count displayed for both players
- [ ] Can view exiled cards (if face-up)
- [ ] Face-down exiled cards show as hidden
- [ ] Visually distinct from graveyard

## Out of Scope

- Exile interactions (casting from exile, etc.)
- Exile effects implementation (engine work)

## Dependencies

- Engine support for exile zone
- Graveyard display pattern established
