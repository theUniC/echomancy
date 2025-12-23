# Mana System

The mana system in Echomancy models the resource production and consumption mechanics of Magic: The Gathering.

## Conceptual Model

### Mana Colors

There are six colors of mana in the game:
- **W** - White mana
- **U** - Blue mana
- **B** - Black mana
- **R** - Red mana
- **G** - Green mana
- **C** - Colorless mana

### Mana Pool

Each player has exactly one mana pool. The pool is a collection of mana organized by color. Each color tracks an integer amount (0 or greater) of mana available to that player.

The mana pool is:
- **Isolated per player** - one player's pool does not affect another's
- **Transient** - mana can be added and spent during gameplay
- **Cleared periodically** - pools are emptied at specific game transitions

## Mana Operations

### Adding Mana

Mana can be added to a player's pool through various game effects:
- Activated abilities (e.g., tapping a land for mana)
- Spell effects
- Triggered abilities

When mana is added:
- The amount must be positive (greater than 0)
- The mana is added to the pool for the specified color
- Multiple additions accumulate

### Spending Mana

Mana can be spent from a player's pool to pay costs:
- Spell casting costs
- Activated ability costs
- Other effects that require mana payment

When mana is spent:
- The amount must be positive (greater than 0)
- The player must have sufficient mana of that color
- If insufficient mana is available, the operation fails
- Spent mana is removed from the pool

### Clearing Mana Pools

Mana pools are cleared (all colors reset to 0) at specific game transitions.

**MVP Behavior:** Mana pools clear when entering the CLEANUP step.

In the full Magic rules, mana empties at the end of each step and phase. This is a known limitation of the MVP that will be addressed in a future update.

## Error Conditions

The mana system enforces the following rules through domain errors:

**InsufficientManaError** - Thrown when attempting to spend more mana than is available in the pool for a specific color.

**InvalidManaAmountError** - Thrown when attempting to add or spend zero or negative mana.

**PlayerNotFoundError** - Thrown when referencing a player that doesn't exist in the game.

## Design Decisions

### Why Isolated Pools?

Each player maintains their own mana pool because mana is a player-specific resource. One player producing mana does not grant that mana to opponents.

### Why Per-Color Tracking?

Mana of different colors cannot be freely substituted. A spell requiring blue mana cannot be paid with red mana. The system tracks each color separately to enforce these restrictions.

### Why Transient State?

Mana pools are part of the game's transient state (not permanent like zones). Mana is produced and consumed within turns and does not persist indefinitely.

## Integration Points

### Game State

Mana pools are initialized when the game starts. Each player's pool begins empty (all colors at 0).

### Step Transitions

Pool clearing is integrated into the step transition system. When the game enters the CLEANUP step, all players' mana pools are automatically cleared.

### Future: Costs System

The mana pool lays the foundation for a full cost evaluation system. Future work will integrate mana costs with spell casting and ability activation, allowing the engine to automatically verify cost payment.

### Future: Mana Abilities

Special abilities that produce mana (called "mana abilities" in Magic rules) will interact with the mana pool. These abilities have unique properties (they don't use the stack) that will be modeled in a future update.

## MVP Limitations

The following are explicitly out of scope for the MVP:

**Not Supported:**
- Mana burn (obsolete rule where unspent mana caused damage)
- Hybrid mana costs (costs that can be paid with either of two colors)
- Phyrexian mana (costs that can be paid with life instead of mana)
- Conditional mana (mana that can only be spent on specific spell types)
- Snow mana
- Mana of any color

**Simplified:**
- Pool clearing happens only at CLEANUP step (not per-step/phase)
- No integration with cost evaluation yet (manual spend calls required)

These limitations are documented with TODO comments in the code and will be addressed in future milestones.

## Testing

The mana system has comprehensive test coverage:
- Pool initialization and isolation
- Adding mana (single color, multiple colors, accumulation)
- Spending mana (sufficient and insufficient scenarios)
- Error conditions (invalid amounts, nonexistent players)
- Pool clearing behavior (CLEANUP step, player isolation)
- Snapshot semantics (returned pools are copies, not live references)

All tests are located in Game.manaPool.test.ts.

## API Reference

For implementation details, see the Game class public methods:
- getManaPool - Query a player's current mana pool
- addMana - Add mana of a specific color to a pool
- spendMana - Spend mana from a pool
- clearManaPool - Clear a specific player's pool
- clearAllManaPools - Clear all players' pools
