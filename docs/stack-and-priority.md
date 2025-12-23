# Stack and Priority

The stack is Magic's mechanism for resolving spells and abilities. The priority system determines who can act and when.

## Stack Overview

The stack operates on **Last In, First Out (LIFO)** order. When the stack resolves:

1. Top item resolves first
2. Each player gets priority after resolution
3. Next item resolves when all players pass
4. Empty stack = priority round ends

## Stack Item Types

### SpellOnStack

Represents a spell that has been cast and is waiting to resolve.

```typescript
type SpellOnStack = {
  kind: "SPELL"
  card: CardInstance
  controllerId: string
  targets: Target[]
}
```

When resolved:
- Spell effect executes
- Card moves to graveyard (instants/sorceries) or battlefield (permanents)
- `SPELL_RESOLVED` event emits
- ETB triggers evaluate (for permanents)

### AbilityOnStack

Represents an activated ability waiting to resolve.

```typescript
type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}
```

**Key differences from spells**:
- Not a spell - doesn't trigger "when you cast a spell"
- Uses Last Known Information - resolves even if source leaves battlefield
- No card movement - just effect execution

### TriggeredAbilityOnStack (Defined, Not Used)

```typescript
type TriggeredAbilityOnStack = {
  kind: "TRIGGERED_ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}
```

**CRITICAL MVP NOTE**: This type is defined but NOT YET USED. Triggered abilities currently execute immediately instead of going on the stack.

**Future behavior**:
- Create `TriggeredAbilityOnStack` when trigger fires
- Add to stack before priority round
- Implement APNAP ordering for simultaneous triggers

## StackItem Union

```typescript
type StackItem = SpellOnStack | AbilityOnStack
// TODO: Add TriggeredAbilityOnStack when implemented
```

## Priority System

Priority determines who can take actions. A player with priority can:

- Cast spells (if timing allows)
- Activate abilities
- Pass priority

### Priority Flow

```
Player A casts spell
    ↓
Spell goes on stack
    ↓
Player B gets priority
    ↓
Player B passes
    ↓
Player A gets priority
    ↓
Player A passes
    ↓
Top of stack resolves
    ↓
Active player gets priority
    ↓
(repeat until stack empty and all pass)
```

### Resolving the Stack

```typescript
// Both players pass priority = top item resolves
game.apply({ type: "PASS_PRIORITY", playerId: opponentId })
game.apply({ type: "PASS_PRIORITY", playerId: controllerId })
// Stack item at top now resolves
```

In tests, use the helper:

```typescript
import { resolveStack } from "./__tests__/helpers"

resolveStack(game, opponentId, playerId)
// Equivalent to both players passing priority
```

## Actions and the Stack

### Actions That Use the Stack

| Action | Stack Item Created |
|--------|-------------------|
| `CAST_SPELL` | `SpellOnStack` |
| `ACTIVATE_ABILITY` | `AbilityOnStack` |

### Actions That Don't Use the Stack

| Action | Effect |
|--------|--------|
| `PLAY_LAND` | Immediate (special action) |
| `ADVANCE_STEP` | Game state change |
| `END_TURN` | Game state change |
| `PASS_PRIORITY` | Priority transfer (may trigger resolution) |
| `DECLARE_ATTACKER` | Combat state change |

## Querying the Stack

```typescript
// Get current stack (readonly)
const stack = game.getStack()

// Check if stack is empty
if (stack.length === 0) {
  // No pending spells/abilities
}

// Inspect top item
const topItem = stack[stack.length - 1]
if (topItem.kind === "SPELL") {
  console.log(`${topItem.card.definition.name} is on top`)
}
```

## Last Known Information

Activated abilities use **Last Known Information**:

```typescript
// Creature with activated ability
game.enterBattlefield(creature, playerId)

// Activate ability - goes on stack
game.apply({ type: "ACTIVATE_ABILITY", playerId, permanentId: creature.instanceId })

// Even if creature is destroyed before resolution...
// The ability still resolves because it uses Last Known Information
```

The `AbilityOnStack.sourceId` captures the source at activation time.

## Stack Resolution Order

When multiple items are on the stack:

```
Bottom: Spell A (cast first)
Middle: Spell B (cast second)
Top:    Ability C (activated third)
```

Resolution order (LIFO):
1. Ability C resolves
2. Spell B resolves
3. Spell A resolves

## MVP Limitations

| Feature | Status |
|---------|--------|
| Triggered abilities on stack | Not implemented |
| APNAP ordering | Not implemented |
| Split second | Not implemented |
| Mana abilities (don't use stack) | Not implemented |
| Targeting validation | Not implemented |
| Counter spells | Structure exists, needs targeting |

## Code Examples

### Casting a Spell

```typescript
// Add spell to hand
const spell = createTestSpell(playerId)
addSpellToHand(game, playerId, spell)

// Cast it (goes on stack)
game.apply({
  type: "CAST_SPELL",
  playerId,
  cardId: spell.instanceId,
  targets: []
})

// Check stack
expect(game.getStack()).toHaveLength(1)
expect(game.getStack()[0].kind).toBe("SPELL")

// Resolve
resolveStack(game, opponentId, playerId)
expect(game.getStack()).toHaveLength(0)
```

### Activating an Ability

```typescript
// Creature with tap ability
const creature: CardInstance = {
  instanceId: "creature-1",
  definition: {
    id: "tapper",
    name: "Tapper",
    types: ["CREATURE"],
    activatedAbility: {
      cost: { type: "TAP" },
      effect: new DrawCardsEffect(1)
    }
  },
  ownerId: playerId
}

game.enterBattlefield(creature, playerId)

// Activate (goes on stack)
game.apply({
  type: "ACTIVATE_ABILITY",
  playerId,
  permanentId: creature.instanceId
})

// Resolve
resolveStack(game, opponentId, playerId)
```

## Source Files

| File | Purpose |
|------|---------|
| `game/StackTypes.ts` | Stack item type definitions |
| `game/Game.ts` | Stack management and resolution |
| `game/GameActions.ts` | Action types including `PASS_PRIORITY` |
