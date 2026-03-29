# UI: Declare Blockers

## Goal

Allow defending player to assign blockers to attackers.

## What We Get When Done

During declare blockers step, defending player can click their creatures and assign them to block attackers.

## Player Experience

1. See "Declare Blockers" indicator
2. Click your creature (highlights as "ready to block")
3. Click an attacking creature to assign block
4. Blocking relationship shown visually (line/arrow)
5. Click "Confirm Blockers" to proceed to damage

## Acceptance Criteria

- [ ] Can assign blockers (click blocker, then attacker)
- [ ] Flying creatures can only be blocked by flying/reach
- [ ] Visual line/arrow shows blocking relationships
- [ ] Confirm button proceeds to damage step

## Dependencies

- Declare attackers (09) working

## Out of Scope

- Combat damage resolution (separate spec)
- Multiple blockers on one attacker
- Combat tricks (instants during combat)
