# Turn Structure

This document describes the phases and steps of a Magic: The Gathering turn as implemented in Echomancy.

## Turn Overview

A turn consists of 5 phases containing 12 steps total:

```
BEGINNING PHASE
├── Untap Step
├── Upkeep Step
└── Draw Step

FIRST MAIN PHASE
└── First Main Step

COMBAT PHASE
├── Beginning of Combat Step
├── Declare Attackers Step
├── Declare Blockers Step
├── Combat Damage Step
└── End of Combat Step

SECOND MAIN PHASE
└── Second Main Step

ENDING PHASE
├── End Step
└── Cleanup Step
```

## Step Constants

Use the `Step` constant object for type-safe step references:

```typescript
import { Step } from "@/echomancy/domainmodel/game/Steps"

Step.UNTAP              // "UNTAP"
Step.UPKEEP             // "UPKEEP"
Step.DRAW               // "DRAW"
Step.FIRST_MAIN         // "FIRST_MAIN"
Step.BEGINNING_OF_COMBAT // "BEGINNING_OF_COMBAT"
Step.DECLARE_ATTACKERS  // "DECLARE_ATTACKERS"
Step.DECLARE_BLOCKERS   // "DECLARE_BLOCKERS"
Step.COMBAT_DAMAGE      // "COMBAT_DAMAGE"
Step.END_OF_COMBAT      // "END_OF_COMBAT"
Step.SECOND_MAIN        // "SECOND_MAIN"
Step.END_STEP           // "END_STEP"
Step.CLEANUP            // "CLEANUP"
```

The type is:

```typescript
type GameSteps = (typeof Step)[keyof typeof Step]
```

## Step Progression

The `StepMachine` handles step advancement:

```typescript
import { advance } from "@/echomancy/domainmodel/game/StepMachine"

const result = advance(Step.FIRST_MAIN)
// result.nextStep === Step.BEGINNING_OF_COMBAT
// result.shouldAdvancePlayer === false

const endResult = advance(Step.CLEANUP)
// endResult.nextStep === Step.UNTAP
// endResult.shouldAdvancePlayer === true
```

### When `shouldAdvancePlayer` is true

The turn passes to the next player when transitioning from CLEANUP to UNTAP.

## Step Order

```typescript
const STEP_ORDER = [
  Step.UNTAP,
  Step.UPKEEP,
  Step.DRAW,
  Step.FIRST_MAIN,
  Step.BEGINNING_OF_COMBAT,
  Step.DECLARE_ATTACKERS,
  Step.DECLARE_BLOCKERS,
  Step.COMBAT_DAMAGE,
  Step.END_OF_COMBAT,
  Step.SECOND_MAIN,
  Step.END_STEP,
  Step.CLEANUP,
]
```

## Step Behaviors

### Untap Step

- Active player's permanents untap
- No priority (players cannot cast spells or activate abilities)
- `STEP_STARTED` event emits

### Upkeep Step

- "At the beginning of your upkeep" triggers check here
- Players get priority
- `STEP_STARTED` event emits

### Draw Step

- Active player draws a card
- Players get priority after draw
- (First player skips draw on first turn - not implemented in MVP)

### Main Phases (First and Second)

- Can play lands (one per turn)
- Can cast sorceries and creatures
- Full priority

### Combat Steps

**Beginning of Combat**:
- "At the beginning of combat" triggers
- Last chance to tap creatures before attacks

**Declare Attackers**:
- Active player declares which creatures attack
- `CREATURE_DECLARED_ATTACKER` event for each attacker
- Creatures tap when attacking (unless vigilance - not implemented)

**Declare Blockers**:
- Defending player assigns blockers
- (Not fully implemented in MVP)

**Combat Damage**:
- Damage is dealt
- (Damage system not implemented in MVP)

**End of Combat**:
- "At end of combat" triggers
- `COMBAT_ENDED` event emits
- Combat state resets

### End Step

- "At the beginning of your end step" triggers
- Last chance to act before cleanup

### Cleanup Step

- Discard to hand size (not implemented in MVP)
- "Until end of turn" effects expire (not implemented)
- No priority normally

## Advancing Steps

### Via Player Action

```typescript
game.apply({ type: "ADVANCE_STEP", playerId: currentPlayerId })
```

### Via Test Helper

```typescript
import { advanceToStep } from "./__tests__/helpers"

advanceToStep(game, Step.FIRST_MAIN)
```

## Landing Played Restrictions

Lands can only be played during main phases:

```typescript
// This will throw InvalidPlayLandStepError if not in main phase
game.apply({ type: "PLAY_LAND", playerId, cardId })
```

One land per turn:

```typescript
// Second land in same turn throws LandLimitExceededError
game.apply({ type: "PLAY_LAND", playerId, cardId: secondLandId })
```

## Spell Timing

MVP implements basic timing:
- Spells can be cast during main phases
- `InvalidCastSpellStepError` thrown if wrong timing

Future: instant-speed casting, flash, etc.

## Extra Phases

The engine supports scheduling extra phases:

```typescript
import { scheduleExtraCombatPhase } from "./__tests__/helpers"

// Schedule an extra combat after this one
scheduleExtraCombatPhase(game)
```

This adds additional steps to the turn after the current step sequence.

## Querying Current Step

```typescript
const currentStep = game.currentStep
const currentPlayer = game.currentPlayerId

if (currentStep === Step.FIRST_MAIN) {
  // Can play lands and cast sorceries
}
```

## Events Emitted

| Step Transition | Event |
|-----------------|-------|
| Any step start | `STEP_STARTED` |
| End of combat | `COMBAT_ENDED` |
| Creature attacks | `CREATURE_DECLARED_ATTACKER` |

## Source Files

| File | Purpose |
|------|---------|
| `game/Steps.ts` | Step constants and type |
| `game/StepMachine.ts` | Step progression logic |
| `game/Game.ts` | Turn management and step transitions |
