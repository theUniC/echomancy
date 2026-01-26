# Deck Assignment Validation

## Overview

When starting a game with deck configurations, validate that ALL players have a deck assigned. Currently, missing decks are silently ignored.

## Problem

If `Game.start()` receives decks but one player's deck is missing, that player silently starts with an empty library while others have 60 cards. This is almost certainly a caller bug, not intentional behavior.

## Solution

Fail-fast: If deck configurations are provided, require entries for ALL players in the game.

## Acceptance Criteria

- [ ] If deck configurations provided, all players must have an entry
- [ ] Throw descriptive error listing which player(s) lack decks
- [ ] Empty deck (`[]`) is allowed (explicit empty vs missing key)
- [ ] No decks at all still works (backward compatibility)

## Out of Scope

- Deck content validation (60-card minimum, 4-of limit)
- Changing `addPlayer()` API
- Mulligan system
