# Cost System

The cost system models payment requirements for casting spells and activating abilities.

## Key Concepts

- **Cost Trait** - All costs implement `can_pay()` and `pay()` methods
- **Cost Context** - Minimal info needed to evaluate costs (player_id, source_id)
- **Atomic Payment** - Multiple costs are fully paid or not paid at all
- **Side-Effect Free Validation** - `can_pay()` never mutates state

## How It Works

See `crates/echomancy-core/src/domain/costs.rs` for implementations.

**Cost Types**:
- **ManaCost**: Pay mana from player's pool. Validates sufficient mana exists, then removes it.
- **TapSelfCost**: Tap the source permanent. Validates untapped and controlled, then marks tapped.
- **SacrificeSelfCost**: Sacrifice the source permanent. Validates on battlefield and controlled, then moves to graveyard (fires "dies" triggers).

**Payment Flow**:
1. Call `can_pay_all_costs()` to validate all costs can be paid
2. If all valid, call `pay_all_costs()` to execute payment in sequence
3. No partial payment (all-or-nothing)
4. After costs paid, effects execute

**Separation from Effects**: Costs are paid first, then effects occur. This matches Magic's rules.

## Rules

- `can_pay()` must be pure (no side effects)
- `pay()` assumes `can_pay()` returned true
- Multiple costs are validated before any are paid
- Payment is atomic (all costs paid or none)
- Costs are paid before effects execute

**Testing**: See cost system tests in `crates/echomancy-core/src/domain/`.
