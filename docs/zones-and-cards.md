# Zones and Cards

The zone system and card model in Echomancy.

## Key Concepts

- **Zones** - Areas where cards exist (Hand, Battlefield, Graveyard, Stack, Library, Exile)
- **CardDefinition** - Shared template/blueprint for all copies of a card
- **CardInstance** - Specific copy in the game with unique ID and owner
- **Zone Transitions** - Cards move between zones via game actions
- **Enter the Battlefield** - Must use `game.enterBattlefield()` to trigger ETB system

## How It Works

### Zone Types

- **HAND** - Player's hand (private to owner)
- **BATTLEFIELD** - In play (public, visible to all)
- **GRAVEYARD** - Discard pile (public)
- **STACK** - Spells and abilities waiting to resolve
- **LIBRARY** - Draw pile (private, ordered)
- **EXILE** - Removed from game zone

Constants available in `src/echomancy/domainmodel/game/Game.ts` via ZoneNames object.

### Player Zones

Each player has isolated zones: hand, battlefield, graveyard. Library and exile exist but less frequently used in MVP.

### Card Definition vs Card Instance

**CardDefinition** is the shared template containing:
- Unique identifier, display name, card types
- Optional spell effect, activated ability, triggered abilities array

**CardInstance** is a specific copy with:
- Unique instance ID (different each game)
- Reference to CardDefinition
- Owner ID

This separation allows multiple copies while maintaining individual identity for zone tracking, tapping, etc.

### Supported Card Types

CREATURE, INSTANT, SORCERY, ARTIFACT, ENCHANTMENT, PLANESWALKER, LAND.

Cards can have multiple types (e.g., Artifact Creature). All permanent types fully supported in MVP (can enter battlefield, have abilities, move between zones, be targeted).

**Note**: Planeswalkers have placeholder state only (no loyalty counters or loyalty abilities in MVP).

### Zone Transitions

**Playing a Land:**
HAND → BATTLEFIELD directly (doesn't use stack).

**Casting a Spell:**
HAND → STACK → Resolution:
- Instants/Sorceries → GRAVEYARD
- Permanents → BATTLEFIELD

**Enter the Battlefield:**
Always use `game.enterBattlefield()`, never push directly to battlefield array. Direct manipulation bypasses ETB trigger system and causes silent bugs.

See `src/echomancy/domainmodel/game/Game.ts` for implementation.

### Zone Change Events

ZONE_CHANGED event emitted when cards move. Enables triggers:
- **ETB** - Destination zone is BATTLEFIELD
- **Dies** - BATTLEFIELD → GRAVEYARD
- **Leaves battlefield** - Source zone is BATTLEFIELD

See `docs/game-events.md` for event details.

## Rules

- Each player has isolated zones (hand, battlefield, graveyard)
- Cards move between zones via game actions only
- Use `game.enterBattlefield()` for all battlefield entry
- CardInstances have unique IDs; CardDefinitions are shared templates
- Cards can have multiple types simultaneously
- Zone change events trigger abilities
