# UI: Activated Abilities

## Goal

Allow players to activate abilities on permanents.

## What We Get When Done

Players can click permanents with activated abilities, see ability options, select one, and activate it (with targeting if needed).

## Player Experience

1. Click permanent with activated ability
2. If multiple abilities, see list to choose from
3. If ability needs targets, targeting flow starts
4. Ability goes on stack
5. Resolves when stack resolves

## Acceptance Criteria

- [ ] Clicking permanent shows available abilities
- [ ] Can select and activate ability
- [ ] Targeting flow works for abilities that need targets
- [ ] Ability appears on stack and resolves

## Dependencies

- Spell casting (08) establishes targeting patterns
- Mana abilities working in engine

## Out of Scope

- Mana payment UI (auto-pay)
- Triggered abilities (automatic)
