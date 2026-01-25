# Mana System

The mana system models resource production and consumption for casting spells and activating abilities.

## Key Concepts

- **Mana Colors** - W (white), U (blue), B (black), R (red), G (green), C (colorless)
- **Mana Pool** - Per-player collection of available mana, organized by color
- **Transient Resource** - Mana is produced and consumed within turns
- **Pool Clearing** - Mana pools empty at specific game transitions

## How It Works

See `src/echomancy/domainmodel/game/Game.ts` for implementation.

**Pool Structure**: Each player has one mana pool tracking integer amounts per color (all start at 0).

**Operations**:
- **Add Mana**: Amount must be positive. Mana accumulates in the specified color.
- **Spend Mana**: Amount must be positive. Player must have sufficient mana of that color. Spent mana is removed from pool.
- **Clear Pool**: All colors reset to 0.

**Clearing Timing**: In MVP, pools clear when entering CLEANUP step. In full Magic rules, mana empties at end of each step/phase (future improvement).

**Errors**:
- `InsufficientManaError`: Not enough mana of required color
- `InvalidManaAmountError`: Zero or negative amount
- `PlayerNotFoundError`: Player doesn't exist

## Rules

- Mana pools are isolated per player
- Mana of different colors cannot be substituted
- Pool amounts are always >= 0
- Pools clear at CLEANUP step (MVP)
- Pools initialize empty (all colors at 0) when game starts

**Testing**: See `Game.manaPool.test.ts` for comprehensive test coverage.
