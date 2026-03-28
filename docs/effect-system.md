# Effect System

The executable part of abilities - what actually happens when an ability resolves.

## Key Concepts

- **Effect Trait** - Single `resolve(game, context)` method
- **EffectContext** - Provides controller_id, source, targets to effect
- **Last Known Information** - Source captured at activation/trigger time
- **Game Methods Only** - Effects use Game methods, never direct mutation
- **Stateless** - Effects don't store instance variables or maintain lifecycle

## How It Works

### Effect Execution

When ability resolves (from stack for activated, immediately for MVP triggers), its Effect executes. Receives game state and execution context, then uses Game methods to produce outcome.

### EffectContext

Provides all information needed to execute effect:

- **controller_id** - Player who controls this ability (always present)
- **source** - Card with this ability (may be None if left battlefield)
- **targets** - Selected targets (always empty in MVP - no targeting system)

### Last Known Information

Source field uses Last Known Information semantics. Captures card state when ability activated/triggered. This means:
- Information remains valid even if source leaves battlefield
- Source may be None if card no longer exists
- For triggers, reflects state at trigger time

**Important**: ETB abilities don't reuse spell targets - permanent entering is new object with own identity.

### Available Game Methods

Effects can use:
- `draw_cards(player_id, count)` - Draw from library
- `enter_battlefield(card, controller_id)` - Put permanent onto battlefield
- `spend_mana(player_id, mana_payment)` - Spend mana from pool

See `crates/echomancy-core/src/domain/game/` for full list.

### Effect Location

New effects go in `crates/echomancy-core/src/domain/effects.rs`.

## Rules

**Must do:**
- Use Game methods for all mutations (draw_cards, enter_battlefield, etc.)
- Use context.controller_id to identify controlling player
- Keep effects stateless

**Must not do:**
- Mutate state directly (no collection pushes, no property assignment)
- Use `game.apply()` (reserved for player actions, not effect resolution)
- Subscribe to events or access external state
- Store instance variables or maintain lifecycle
