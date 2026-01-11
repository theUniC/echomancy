# UI Phase 1c: Battlefield Display

## Goal

Display cards on the battlefield for both players, showing card names, tapped state, and creature stats.

## What We Get When Done

The `/games/[gameId]` page shows:
- Your battlefield permanents
- Opponent's battlefield permanents
- Tapped/untapped indication
- Creature power/toughness
- Basic card information

## Status

**Placeholder spec** - Will be fully detailed after Phase 1b is complete.

## Architectural Decision: PixiJS Introduction

**Phase 1c is where PixiJS is introduced to the project.**

### Why PixiJS in Phase 1c?

- Cards are the core visual element of a TCG
- Future phases will need drag & drop, animations, complex interactions
- Introducing PixiJS now avoids migration/rewrite later
- PixiJS handles many cards on screen efficiently

### Rendering Strategy

**PixiJS for cards:**
- Battlefield cards
- Hand cards (Phase 1d)
- All card rendering uses PixiJS sprites/containers

**HTML/CSS for UI chrome:**
- Turn/phase display (Phase 1b)
- Life totals (Phase 1b)
- Buttons and controls (Phase 2+)
- Static game information

This hybrid approach uses PixiJS where it provides value (cards, animations) and HTML/CSS for standard UI elements.

## Dependencies

- Phase 1a complete (route and GameSnapshot working)
- Phase 1b complete (basic game info displayed)
- **PixiJS 8+ installed** (add to package.json)
- **@pixi/react or react-pixi** for React integration

## Open Questions

- PixiJS integration pattern (which library: @pixi/react vs react-pixi vs custom)
- Card sprite design (asset pipeline, card template rendering)
- Tapped indication (sprite rotation, overlay, shader)
- Card layout algorithm (spacing, overlapping, z-index)
- Grouping strategy (by type, by controller)
- How to display non-creature permanents visually distinct from creatures

These questions will be answered during brainstorming after Phase 1b implementation.
