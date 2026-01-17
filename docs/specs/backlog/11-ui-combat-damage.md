# UI: Combat Damage

## Goal

Resolve combat damage with visual feedback.

## What We Get When Done

After blockers confirmed, damage resolves automatically with clear visual feedback. Life totals update, creatures die and go to graveyard.

## Player Experience

1. After blockers confirmed, damage step begins
2. Damage numbers flash on creatures
3. Life totals update for unblocked damage
4. Creatures with lethal damage fade out and move to graveyard
5. Combat ends, return to main phase

## Acceptance Criteria

- [ ] Damage resolves automatically after blockers
- [ ] Life totals update for unblocked attackers
- [ ] Creatures die when damage >= toughness
- [ ] Dead creatures visually move to graveyard

## Dependencies

- Declare blockers (10) working
- Combat system working in engine

## Out of Scope

- First strike / double strike
- Damage assignment order (multiple blockers)
- Trample
