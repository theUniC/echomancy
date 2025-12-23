# Stack and Priority

The stack is Magic's mechanism for resolving spells and abilities. The priority system determines who can act and when.

## Stack Overview

The stack operates on Last In, First Out (LIFO) order. Items are added to the top and resolve from the top down.

When the stack resolves:
1. Top item resolves first
2. Each player gets priority after resolution
3. Next item resolves when all players pass priority
4. Empty stack means the priority round ends

## Stack Item Types

### Spells on Stack

When a player casts a spell, it goes on the stack. The stack item contains the card being cast, the controller, and any targets (empty in MVP).

When a spell resolves:
- The spell's effect executes
- The card moves to graveyard (instants/sorceries) or battlefield (permanents)
- A SPELL_RESOLVED event emits
- ETB triggers evaluate for permanents

### Abilities on Stack

When a player activates an ability, it goes on the stack separately from the source permanent. The stack item contains the source permanent's ID, the effect to execute, the controller, and any targets.

Key differences from spells:
- Not a spell - doesn't trigger "when you cast a spell" effects
- Uses Last Known Information - resolves even if source leaves battlefield
- No card movement - just effect execution

### Triggered Abilities on Stack (Not Yet Implemented)

A TriggeredAbilityOnStack type is defined but not yet used. In the MVP, triggered abilities execute immediately instead of going on the stack.

Future behavior will:
- Create a stack item when a trigger fires
- Add it to the stack before the priority round
- Implement APNAP ordering for simultaneous triggers
- Allow players to respond to triggered abilities

## Priority System

Priority determines who can take actions. A player with priority can:
- Cast spells (if timing allows)
- Activate abilities
- Pass priority

### Priority Flow

When a player takes an action that uses the stack:
1. The item goes on the stack
2. The active player gets priority first
3. Each player can respond or pass
4. When all players pass in succession, the top item resolves
5. Active player gets priority again
6. Repeat until stack is empty and all players pass

### Resolution

Both players passing priority in succession causes the top stack item to resolve. After resolution, the active player gets priority again.

## Actions and the Stack

**Actions that use the stack:**
- Casting spells (CAST_SPELL)
- Activating abilities (ACTIVATE_ABILITY)

**Actions that don't use the stack:**
- Playing lands (special action, no stack)
- Advancing to the next step
- Ending the turn
- Passing priority
- Declaring attackers (combat state change)

## Last Known Information

Activated abilities use Last Known Information. When an ability goes on the stack, it captures information about its source. Even if the source permanent is destroyed before resolution, the ability still resolves using the captured information.

This is different from spells, where the card itself is on the stack.

## Stack Resolution Order

With multiple items on the stack, they resolve in reverse order from how they were added. If Spell A is cast, then Spell B, then Ability C is activated, the resolution order is: Ability C first, then Spell B, then Spell A.

## MVP Limitations

| Feature | Status |
|---------|--------|
| Triggered abilities on stack | Not implemented |
| APNAP ordering | Not implemented |
| Mana abilities (don't use stack) | Not implemented |
| Targeting validation | Not implemented |
| Counter spells | Structure exists, needs targeting |
| Split second | Not implemented |
