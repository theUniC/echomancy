# Cost System

The cost system models payment requirements for casting spells and activating abilities.

## Key Concepts

- **Cost Interface** - All costs implement `canPay()` and `pay()` methods
- **Cost Context** - Minimal info needed to evaluate costs (playerId, sourceId)
- **Atomic Payment** - Multiple costs are fully paid or not paid at all
- **Side-Effect Free Validation** - `canPay()` never mutates state

## How It Works

See `src/echomancy/domainmodel/costs/` for implementations.

**Cost Types**:
- **ManaCost**: Pay mana from player's pool. Validates sufficient mana exists, then removes it.
- **TapSelfCost**: Tap the source permanent. Validates untapped and controlled, then marks tapped.
- **SacrificeSelfCost**: Sacrifice the source permanent. Validates on battlefield and controlled, then moves to graveyard (fires "dies" triggers).

**Payment Flow**:
1. Call `canPayAllCosts()` to validate all costs can be paid
2. If all valid, call `payAllCosts()` to execute payment in sequence
3. No partial payment (all-or-nothing)
4. After costs paid, effects execute

**Separation from Effects**: Costs are paid first, then effects occur. This matches Magic's rules.

## Rules

- `canPay()` must be pure (no side effects)
- `pay()` assumes `canPay()` returned true
- Multiple costs are validated before any are paid
- Payment is atomic (all costs paid or none)
- Costs are paid before effects execute

**Testing**: See `Game.costs.test.ts` for comprehensive test coverage.

**Implementation Files**:
- `src/echomancy/domainmodel/costs/Cost.ts` - Interface
- `src/echomancy/domainmodel/costs/impl/ManaCost.ts`
- `src/echomancy/domainmodel/costs/impl/TapSelfCost.ts`
- `src/echomancy/domainmodel/costs/impl/SacrificeSelfCost.ts`
