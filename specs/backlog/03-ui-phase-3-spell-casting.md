# UI Phase 3: Spell Casting with Targets

## Goal

Cast spells that require targets.

## What We Get When Done

Players can click spells in hand, see valid targets highlighted, select targets, and confirm or cancel. Spells go on the stack with their targets and resolve correctly.

## Player Experience

1. Click spell in hand
2. Valid targets highlight (glow, border, or other indicator)
3. Click target to select it
4. Confirm button appears
5. Click confirm -> spell goes on stack
6. Or click cancel -> selection cleared, back to normal

## Targeting Flow

```
[Click spell] -> [Targets highlight] -> [Click target] -> [Confirm/Cancel]
                                              |
                                              v
                                    [Spell on stack with target]
```

## Visual Feedback

- Spell being cast: visually distinct (raised, glowing)
- Valid targets: highlighted border or glow
- Selected target: stronger highlight, checkmark, or similar
- Invalid targets: dimmed or no highlight
- Confirm/Cancel buttons appear during targeting

## Acceptance Criteria

- [ ] Clicking spell in hand initiates casting flow
- [ ] Only valid targets are highlighted
- [ ] Clicking invalid area does nothing (or shows error)
- [ ] Can select required number of targets
- [ ] Can cancel mid-selection (returns to normal state)
- [ ] Confirming puts spell on stack with selected targets
- [ ] Spell resolves and affects the correct target
- [ ] Mana is paid (taps lands or uses mana pool)

## Dependencies

- Phase 2 complete (basic actions working)
- Mana system must be working in engine

## Out of Scope

- Combat
- Activated abilities on permanents
- Multiple targets (single target only for MVP)
