# UI Phase 4: Combat

## Goal

Full combat flow with attackers and blockers.

## What We Get When Done

Active player can select attackers, defending player can assign blockers, damage resolves automatically, creatures die and go to graveyard.

## Combat Flow

1. **Declare Attackers Step**
   - Active player clicks creatures to toggle "attacking" state
   - Selected attackers show visual indicator (sword icon, red border, etc.)
   - Click "Confirm Attackers" to proceed

2. **Declare Blockers Step**
   - Defending player clicks their creature, then clicks an attacker to block
   - Blocking relationship shown visually (line, arrow, grouping)
   - Click "Confirm Blockers" to proceed

3. **Damage Step**
   - Damage resolves automatically
   - Life totals update
   - Creatures with lethal damage go to graveyard

## Visual Indicators

| State | Visual |
|-------|--------|
| Can attack (untapped creature) | Subtle highlight when hovering |
| Selected as attacker | Red border or sword icon |
| Attacking (confirmed) | Moved forward or arrow pointing at opponent |
| Can block | Subtle highlight when in blockers step |
| Blocking | Line/arrow connecting to attacker |
| Taking damage | Flash red, number shown |
| Dying | Fade out, move to graveyard |

## Player Experience - Attacking

1. Enter combat (automatic or button)
2. See "Declare Attackers" indicator
3. Click creatures to toggle attack selection
4. Click "Confirm Attackers"
5. Wait for blocker declaration

## Player Experience - Blocking

1. See "Declare Blockers" indicator
2. Click your creature (it highlights as "ready to block")
3. Click an attacking creature to assign block
4. Repeat for other blockers
5. Click "Confirm Blockers"
6. Damage happens automatically

## Acceptance Criteria

- [ ] Can select multiple attackers by clicking
- [ ] Can deselect attacker by clicking again
- [ ] Tapped creatures cannot be selected as attackers
- [ ] Can confirm attackers to proceed to blockers step
- [ ] Can assign blockers (click blocker, then attacker)
- [ ] Flying creatures can only be blocked by flying/reach
- [ ] Can confirm blockers to proceed to damage
- [ ] Combat damage updates life totals correctly
- [ ] Creatures die when damage >= toughness
- [ ] Dead creatures move to graveyard
- [ ] Unblocked attackers deal damage to defending player

## Dependencies

- Phase 3 complete (spells and targeting working)
- Combat system working in engine

## Out of Scope

- First strike / double strike
- Multiple blockers on one attacker
- Damage assignment order
- Combat tricks (instants during combat)
