# Effect System

Effects are the executable part of abilities - what actually happens when an ability resolves.

## Overview

When an ability resolves (from the stack or immediately for MVP triggers), its **Effect** is executed. Effects receive the game state and an execution context, then use Game methods to produce the desired outcome.

## Effect Interface

```typescript
interface Effect {
  resolve(game: Game, context: EffectContext): void
}
```

All effects must implement this interface. The `resolve` method is called when the ability resolves.

## Effect Context

The `EffectContext` provides all information needed to execute an effect:

```typescript
type EffectContext = {
  /** Card with this ability (may be undefined - Last Known Information) */
  source?: CardInstance

  /** Player who controls this ability (always present) */
  controllerId: string

  /** Selected targets (MVP: always empty) */
  targets: Target[]
}
```

### Last Known Information

The `source` field uses Last Known Information semantics:
- Captures the card state when the ability was activated/triggered
- Remains valid even if the source card leaves the battlefield
- May be `undefined` if the card no longer exists

### Important Notes

- **ETB abilities do NOT reuse spell targets** - the permanent entering is a new object
- **`controllerId` is always present** - use this to identify who controls the effect
- **Targets are always empty in MVP** - no targeting system implemented yet

## Implementation Rules

### MUST Do

```typescript
// Use Game methods for all mutations
game.drawCards(context.controllerId, 1)
game.enterBattlefield(permanent, context.controllerId)
game.dealDamage(targetId, amount)  // future

// Use context.controllerId for the controlling player
const player = game.getCurrentPlayer(context.controllerId)
```

### MUST NOT Do

```typescript
// WRONG: Direct state mutation
playerState.hand.cards.push(card)

// WRONG: Using game.apply() in effects
game.apply({ type: "PLAY_LAND", playerId, cardId })

// WRONG: Event subscription
game.on('zoneChanged', () => { ... })

// WRONG: Instance variables or lifecycle
class MyEffect implements Effect {
  private counter = 0  // NO - effects are stateless
}
```

## Creating Effects

### Simple Effect (Function-based in Triggers)

For triggers, effects are typically inline functions:

```typescript
const trigger: Trigger = {
  eventType: GameEventTypes.ZONE_CHANGED,
  condition: (game, event, source) => /* ... */,
  effect: (game, context) => {
    game.drawCards(context.controllerId, 1)
  }
}
```

### Reusable Effect (Class-based)

For activated abilities or reusable effects, create a class:

```typescript
// src/echomancy/domainmodel/effects/impl/DrawCardsEffect.ts
import type { Effect } from "../Effect"
import type { Game } from "../../game/Game"
import type { EffectContext } from "../EffectContext"

export class DrawCardsEffect implements Effect {
  constructor(private readonly count: number) {}

  resolve(game: Game, context: EffectContext): void {
    game.drawCards(context.controllerId, this.count)
  }
}
```

Usage:

```typescript
const activatedAbility: ActivatedAbility = {
  cost: { type: "TAP" },
  effect: new DrawCardsEffect(2)
}
```

## Effect Location Convention

Place new effect implementations in:

```
src/echomancy/domainmodel/effects/impl/
```

## Available Game Methods for Effects

| Method | Purpose |
|--------|---------|
| `game.drawCards(playerId, count)` | Draw cards from library |
| `game.enterBattlefield(card, controllerId)` | Put permanent onto battlefield |
| `game.dealDamage(targetId, amount)` | Deal damage (future) |
| `game.adjustLifeTotal(playerId, amount)` | Change life total (via Player) |

## MVP Limitations

The following are not supported in MVP:

| Feature | Status |
|---------|--------|
| Targeting | Not implemented - `targets` always empty |
| Duration tracking | No "until end of turn" effects |
| Modal effects | No "Choose one" |
| Damage system | No `dealDamage()` yet |
| Token creation | Not implemented |
| Counter manipulation | Not implemented |

## Examples

### Draw Cards Effect

```typescript
class DrawCardsEffect implements Effect {
  constructor(private readonly count: number) {}

  resolve(game: Game, context: EffectContext): void {
    game.drawCards(context.controllerId, this.count)
  }
}
```

### Conditional Effect (in trigger)

```typescript
effect: (game, context) => {
  // Only draw if controller has less than 7 cards
  const hand = game.getPlayerState(context.controllerId).hand
  if (hand.cards.length < 7) {
    game.drawCards(context.controllerId, 1)
  }
}
```

### Effect Using Source Card

```typescript
effect: (game, context) => {
  if (context.source) {
    // Do something based on the source card
    const sourceName = context.source.definition.name
    console.log(`${sourceName}'s ability resolves`)
  }
  game.drawCards(context.controllerId, 1)
}
```

## Source Files

| File | Purpose |
|------|---------|
| `effects/Effect.ts` | Effect interface definition |
| `effects/EffectContext.ts` | Execution context type |
| `effects/impl/` | Concrete effect implementations |
