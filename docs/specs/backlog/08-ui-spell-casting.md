# UI: Spell Casting with Targets

## Goal

Cast spells that require targets.

## What We Get When Done

Players can click spells in hand, see valid targets highlighted, select a target, and confirm. Spell goes on stack and resolves.

## Player Experience

1. Click spell in hand
2. Valid targets highlight
3. Click target to select
4. Confirm button appears
5. Click confirm → spell goes on stack
6. Or click cancel → back to normal

## Acceptance Criteria

- [ ] Clicking spell initiates casting flow
- [ ] Only valid targets are highlighted
- [ ] Can cancel mid-selection
- [ ] Confirming puts spell on stack with target
- [ ] Spell resolves affecting correct target

## Dependencies

- Priority controls (07) working
- Mana system working in engine

## Out of Scope

- Mana payment UI (uses auto-pay for now)
- Multiple targets (single target only)
- Combat
- Activated abilities
