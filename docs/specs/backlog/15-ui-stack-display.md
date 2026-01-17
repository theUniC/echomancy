# UI: Stack Display

## Overview

Display the stack when spells or abilities are waiting to resolve.

## User Stories

**As a player**, I want to:
- See what spells/abilities are on the stack
- Understand the order they will resolve (LIFO)
- See targets of spells/abilities

## Player Experience

- Stack appears when not empty
- Shows spells/abilities in resolution order (top = resolves first)
- Each item shows: name, controller, target(s)

## Game Rules & Mechanics

- Stack resolves Last-In-First-Out (LIFO)
- Both players can see stack contents (public information)
- Stack is empty most of the time (only shows during spell/ability resolution)

## Acceptance Criteria

- [ ] Stack is hidden when empty
- [ ] Stack shows all pending spells/abilities
- [ ] Order is correct (top = next to resolve)
- [ ] Targets are visible
- [ ] Controller is indicated

## Out of Scope

- Responding to stack (adding spells) - that's spell casting spec
- Stack manipulation effects

## Dependencies

- Spell casting (spec 08) working
- Priority system (spec 07) working
