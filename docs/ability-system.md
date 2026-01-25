# Ability System

How cards produce effects through activated and triggered abilities.

## Key Concepts

- **Abilities are Declarative** - Describe what should happen, not how
- **Activated Abilities** - Player-initiated, cost-based
- **Triggered Abilities** - Automatic when trigger condition met
- **Last Known Information** - Abilities resolve even if source leaves battlefield
- **Game Evaluates** - Game checks triggers at explicit evaluation points

## How It Works

### Activated Abilities

Player-initiated abilities that require cost payment and go on the stack.

**Activation flow:**
1. Player has priority
2. Player pays cost (MVP: tap cost only)
3. Ability item created and put on stack
4. Other players get priority to respond
5. Stack resolves, effect executes

**Last Known Information**: If source permanent leaves battlefield before resolution, ability still resolves using information captured at activation time.

### Triggered Abilities

Automatic abilities that fire when trigger condition is met.

**Trigger flow:**
1. Game event occurs (creature enters battlefield, attacks, etc.)
2. Game evaluates all triggers on all permanents
3. Triggers whose conditions match the event fire
4. Trigger's effect executes

**Trigger structure:**
- **Event type** - What kind of game event activates trigger
- **Condition** - Pure predicate that must return true to fire
- **Effect** - What happens when trigger fires

Condition is pure function with no side effects. Receives game state, event, source card; returns boolean.

See `src/echomancy/domainmodel/game/Game.ts` for trigger evaluation implementation.

### Type Guards

- `isActivatedAbility()` - Checks for cost and effect properties
- `isTrigger()` - Checks for eventType and condition properties

Located in ability type definitions.

### Adding Abilities to Cards

Cards can have:
- `activatedAbility` property - Single activated ability
- `triggers` array - Multiple triggered abilities

MVP doesn't support multiple activated abilities per card.

## Rules

- Effects must use Game methods for all state mutations
- Never use `game.apply()` inside an effect (reserved for player actions)
- Trigger conditions must be pure functions
- Abilities don't store mutable state
- Use constants (GameEventTypes, ZoneNames) instead of string literals
- Game evaluates triggers at explicit points only (no continuous evaluation)

