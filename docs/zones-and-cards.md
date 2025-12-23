# Zones and Cards

This document describes the zone system and card model in Echomancy.

## Zones

Zones are areas where cards exist during the game. Each player has private zones, and some zones are shared.

### Zone Types

```typescript
type ZoneName =
  | "HAND"        // Player's hand (private)
  | "BATTLEFIELD" // In play (public)
  | "GRAVEYARD"   // Discard pile (public)
  | "STACK"       // Spells/abilities waiting to resolve
  | "LIBRARY"     // Draw pile (private, ordered)
  | "EXILE"       // Removed from game
```

### Zone Constants

Use `ZoneNames` to avoid magic strings:

```typescript
import { ZoneNames } from "@/echomancy/domainmodel/zones/Zone"

ZoneNames.HAND        // "HAND"
ZoneNames.BATTLEFIELD // "BATTLEFIELD"
ZoneNames.GRAVEYARD   // "GRAVEYARD"
ZoneNames.STACK       // "STACK"
ZoneNames.LIBRARY     // "LIBRARY"
ZoneNames.EXILE       // "EXILE"
```

### Zone Structure

```typescript
type Zone = {
  cards: CardInstance[]
}
```

### Player State

Each player has their own zones:

```typescript
type PlayerState = {
  hand: Zone
  battlefield: Zone
  graveyard: Zone
}
```

## Cards

### CardDefinition

The template/blueprint for a card. Shared across all instances of that card.

```typescript
type CardDefinition = {
  id: string                         // Unique card ID
  name: string                       // Display name
  types: CardType[]                  // Card types
  effect?: Effect                    // Spell effect (for instants/sorceries)
  activatedAbility?: ActivatedAbility // Activated ability
  triggers?: Trigger[]               // Triggered abilities
}
```

### CardType

```typescript
type CardType =
  | "CREATURE"
  | "INSTANT"
  | "SORCERY"
  | "ARTIFACT"
  | "ENCHANTMENT"
  | "PLANESWALKER"
  | "LAND"
```

### CardInstance

A specific instance of a card in the game. Has a unique ID and tracks ownership.

```typescript
type CardInstance = {
  instanceId: string      // Unique per-game instance
  definition: CardDefinition
  ownerId: string         // Player who owns this card
}
```

## Creating Cards

### Simple Creature

```typescript
const creature: CardInstance = {
  instanceId: "creature-1",
  definition: {
    id: "grizzly-bears",
    name: "Grizzly Bears",
    types: ["CREATURE"]
  },
  ownerId: playerId
}
```

### Instant Spell

```typescript
const instant: CardInstance = {
  instanceId: "spell-1",
  definition: {
    id: "giant-growth",
    name: "Giant Growth",
    types: ["INSTANT"],
    effect: new GrowthEffect(3, 3)  // hypothetical effect
  },
  ownerId: playerId
}
```

### Creature with ETB Trigger

```typescript
const elvishVisionary: CardInstance = {
  instanceId: "visionary-1",
  definition: {
    id: "elvish-visionary",
    name: "Elvish Visionary",
    types: ["CREATURE"],
    triggers: [{
      eventType: GameEventTypes.ZONE_CHANGED,
      condition: (game, event, source) =>
        event.card.instanceId === source.instanceId &&
        event.toZone === ZoneNames.BATTLEFIELD,
      effect: (game, context) =>
        game.drawCards(context.controllerId, 1)
    }]
  },
  ownerId: playerId
}
```

### Creature with Activated Ability

```typescript
const tapper: CardInstance = {
  instanceId: "tapper-1",
  definition: {
    id: "prodigal-sorcerer",
    name: "Prodigal Sorcerer",
    types: ["CREATURE"],
    activatedAbility: {
      cost: { type: "TAP" },
      effect: new DealDamageEffect(1)  // hypothetical
    }
  },
  ownerId: playerId
}
```

### Land

```typescript
const land: CardInstance = {
  instanceId: "land-1",
  definition: {
    id: "forest",
    name: "Forest",
    types: ["LAND"]
  },
  ownerId: playerId
}
```

## Zone Transitions

### Playing a Land

```
HAND → BATTLEFIELD
```

```typescript
game.apply({ type: "PLAY_LAND", playerId, cardId })
```

### Casting a Spell

```
HAND → STACK → BATTLEFIELD (permanent)
                or
HAND → STACK → GRAVEYARD (instant/sorcery)
```

```typescript
// Goes to stack
game.apply({ type: "CAST_SPELL", playerId, cardId, targets: [] })

// After resolution, moves to final zone
resolveStack(game, opponentId, playerId)
```

### Enter the Battlefield

Always use `game.enterBattlefield()`:

```typescript
// CORRECT
game.enterBattlefield(card, controllerId)

// WRONG - bypasses ETB triggers
playerState.battlefield.cards.push(card)
```

## Querying Zones

```typescript
// Get player state
const playerState = game.getPlayerState(playerId)

// Query hand
const cardsInHand = playerState.hand.cards

// Query battlefield
const permanents = playerState.battlefield.cards

// Find specific card
const creature = permanents.find(c => c.instanceId === creatureId)

// Filter by type
const creatures = permanents.filter(c =>
  c.definition.types.includes("CREATURE")
)

// Check if card exists
const hasCard = cardsInHand.some(c => c.instanceId === cardId)
```

## Zone Change Events

When cards change zones, a `ZONE_CHANGED` event is emitted:

```typescript
type ZoneChangedEvent = {
  type: "ZONE_CHANGED"
  card: CardInstance
  fromZone: ZoneName
  toZone: ZoneName
  controllerId: string
}
```

This allows triggers like:

- **ETB**: `toZone === "BATTLEFIELD"`
- **Dies**: `fromZone === "BATTLEFIELD" && toZone === "GRAVEYARD"`
- **Leaves battlefield**: `fromZone === "BATTLEFIELD"`

## Test Helpers

```typescript
import {
  createTestCreature,
  createTestSpell,
  addCreatureToBattlefield,
  addSpellToHand,
  createElvishVisionary
} from "./__tests__/helpers"

// Create and add creature to battlefield
const creature = createTestCreature(playerId)
addCreatureToBattlefield(game, playerId, creature)

// Create and add spell to hand
const spell = createTestSpell(playerId)
addSpellToHand(game, playerId, spell)

// Create themed creature
const visionary = createElvishVisionary(playerId, () => {
  console.log("ETB triggered!")
})
```

## Multiple Card Types

A card can have multiple types:

```typescript
const artifactCreature: CardDefinition = {
  id: "steel-golem",
  name: "Steel Golem",
  types: ["ARTIFACT", "CREATURE"]
}
```

Check with:

```typescript
const isCreature = card.definition.types.includes("CREATURE")
const isArtifact = card.definition.types.includes("ARTIFACT")
const isArtifactCreature = isCreature && isArtifact
```

## Source Files

| File | Purpose |
|------|---------|
| `zones/Zone.ts` | Zone types and constants |
| `cards/CardDefinition.ts` | Card template type |
| `cards/CardInstance.ts` | Runtime card instance |
| `game/PlayerState.ts` | Player zone structure |
