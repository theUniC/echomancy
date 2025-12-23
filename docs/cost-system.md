# Cost System

The cost system models the payment requirements for casting spells and activating abilities in Echomancy.

## Overview

Costs are domain objects that represent resources a player must pay to perform game actions. The cost system provides explicit validation and atomic payment mechanisms, ensuring costs are either fully paid or not paid at all.

## Core Concepts

### Cost Interface

All costs implement a common interface with two methods:

- **canPay**: Validates whether a cost can be paid given the current game state. This method must be side-effect free and never mutate state.
- **pay**: Executes the cost payment, mutating game state. This method assumes canPay has returned true.

### Cost Context

The CostContext provides the minimal information needed to evaluate and pay costs:

- **playerId**: The player who is paying the cost
- **sourceId**: The card or ability that requires the cost

### Atomic Payment

Multiple costs can be combined on a single spell or ability. The system ensures atomicity through two helper functions:

- **canPayAllCosts**: Validates that all costs can be paid before paying any
- **payAllCosts**: Pays all costs in sequence

If any cost cannot be paid, nothing happens - no partial payment occurs.

## Supported Cost Types

### Mana Cost

Pay mana from the player's mana pool.

**Example**: A cost of `{ G: 2, W: 1 }` requires 2 green mana and 1 white mana.

**Validation**: Checks that the player has sufficient mana of each required color in their pool.

**Payment**: Removes the specified mana amounts from the player's pool.

### Tap Self Cost

Tap the permanent that has the ability being activated.

**Validation**: Checks that:
- The permanent exists on the battlefield
- The permanent is untapped
- The permanent is controlled by the player

**Payment**: Marks the permanent as tapped.

**MVP Limitation**: Only creatures track tapped state. Non-creatures are assumed to always be untapped. This will be expanded in the future to track tap state for all permanents.

### Sacrifice Self Cost

Sacrifice the permanent that has the ability being activated.

**Validation**: Checks that:
- The permanent exists on the battlefield
- The permanent is controlled by the player

**Payment**: Moves the permanent from the battlefield to the graveyard.

**MVP Limitation**: Does not emit ZONE_CHANGED events or trigger abilities that fire on sacrifice. These will be added in a future update.

## Integration with Abilities

Costs are separate from effects. When a player activates an ability or casts a spell:

1. All costs are validated using canPay
2. If all costs can be paid, they are paid using pay
3. After all costs are paid, effects execute

This separation ensures costs are always paid before effects occur, matching Magic's rules.

## Error Handling

The cost system uses specific domain errors:

- **PermanentNotFoundError**: The permanent referenced in the cost does not exist
- **PermanentAlreadyTappedError**: Attempting to tap a permanent that is already tapped
- **PermanentNotControlledError**: The player does not control the permanent being used for the cost
- **InsufficientManaError**: The player does not have enough mana to pay the cost

These errors provide clear feedback about why a cost cannot be paid.

## MVP Limitations

The following cost features are not supported in the current implementation:

**Not Supported:**
- Alternative costs
- Cost reductions or cost modification effects
- X costs (variable costs)
- Hybrid mana costs
- Phyrexian mana costs
- Costs that tap or sacrifice other permanents (only "self" is supported)
- Costs that require multiple permanents
- Conditional costs based on permanent properties

**Simplified:**
- Only creatures track tap state (artifacts, lands, and enchantments are assumed untapped)
- Sacrifice does not trigger abilities or emit events

These limitations are documented in the code with TODO comments and will be addressed in future updates.

## Implementation Details

For implementation specifics, see:

- Cost interface: `src/echomancy/domainmodel/costs/Cost.ts`
- Mana cost: `src/echomancy/domainmodel/costs/impl/ManaCost.ts`
- Tap cost: `src/echomancy/domainmodel/costs/impl/TapSelfCost.ts`
- Sacrifice cost: `src/echomancy/domainmodel/costs/impl/SacrificeSelfCost.ts`
- Test suite: `src/echomancy/domainmodel/game/__tests__/Game.costs.test.ts`
