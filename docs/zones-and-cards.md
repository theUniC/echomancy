# Zones and Cards

This document describes the zone system and card model in Echomancy.

## Zones

Zones are areas where cards exist during the game.

### Zone Types

- **HAND:** Player's hand (private to owner)
- **BATTLEFIELD:** In play (public, visible to all)
- **GRAVEYARD:** Discard pile (public)
- **STACK:** Spells and abilities waiting to resolve
- **LIBRARY:** Draw pile (private, ordered)
- **EXILE:** Removed from game zone

### Zone Constants

The ZoneNames object provides constants for all zone names. Use these instead of string literals for type safety.

### Player Zones

Each player has their own hand, battlefield, and graveyard. The library and exile zones also exist but are less frequently manipulated in the MVP.

## Cards

### Card Definition vs Card Instance

**CardDefinition** is the template or blueprint for a card. It's shared across all copies of that card and contains:
- Unique identifier
- Display name
- Card types (creature, instant, etc.)
- Optional spell effect
- Optional activated ability
- Optional triggered abilities array

**CardInstance** is a specific instance of a card in the game. Each instance has:
- Unique instance ID (different each game)
- Reference to its CardDefinition
- Owner ID (the player who owns this card)

This separation allows multiple copies of the same card while each maintains its own identity for tracking zones, tapping, etc.

### Card Types

The supported card types are: CREATURE, INSTANT, SORCERY, ARTIFACT, ENCHANTMENT, PLANESWALKER, and LAND.

A card can have multiple types (e.g., Artifact Creature).

## Zone Transitions

### Playing a Land

When a land is played, it moves from HAND directly to BATTLEFIELD. This is a special action that doesn't use the stack.

### Casting a Spell

When a spell is cast, it moves from HAND to STACK. After resolution:
- Instants and sorceries go to GRAVEYARD
- Permanents (creatures, artifacts, enchantments, planeswalkers) go to BATTLEFIELD

### Enter the Battlefield

When a permanent enters the battlefield, it must go through the `game.enterBattlefield()` method. This is critical because:
- It properly initializes creature state (tapped status, attack history)
- It emits the ZONE_CHANGED event
- It triggers ETB (enter the battlefield) abilities

Never push directly to the battlefield array - this bypasses the ETB system and will cause silent bugs.

## Zone Change Events

When cards change zones, a ZONE_CHANGED event is emitted. This enables triggers like:
- **ETB:** Fires when destination zone is BATTLEFIELD
- **Dies:** Fires when moving from BATTLEFIELD to GRAVEYARD
- **Leaves battlefield:** Fires when source zone is BATTLEFIELD

## Multiple Card Types

A card can have multiple types. To check if a card is a creature, check if the types array includes "CREATURE". A card can be both an Artifact and a Creature simultaneously.
