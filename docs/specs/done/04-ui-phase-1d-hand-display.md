# UI Phase 1d: Hand Display

## Overview

Display hand cards for the viewing player and opponent hand count. This is a read-only display - no interactions.

## User Stories

**As a player**, I want to:
- See all cards in my hand so I can plan my strategy
- Know how many cards my opponent has without seeing their details

## Player Experience

### What the Player Sees

**Your Hand (bottom of screen):**
- All your hand cards visible
- Each card shows: name, type, and stats (if creature)
- Cards readable without interaction

**Opponent Hand:**
- Card count only (e.g., "Opponent's Hand: 5 cards")
- No individual cards shown (hidden information)

### Player Flow

1. Navigate to game page
2. See battlefield (Phase 1c)
3. Look down to see your hand cards
4. Look up to see opponent's card count

## Game Rules & Mechanics

### Visibility Rules

- **Your Hand**: Full visibility (name, type, P/T, keywords)
- **Opponent Hand**: Count only (hidden information per MTG rules)

### Card Ordering

- Cards shown in order received (no sorting for MVP)

## Acceptance Criteria

- [x] All cards in hand are visible with correct info
- [x] Opponent hand count is displayed correctly
- [x] Singular/plural: "1 card" vs "X cards"
- [x] Empty hand shows empty zone (no crash)
- [x] Consistent visual style with battlefield (Phase 1c)

## Out of Scope

- Card interactions (click, hover, drag)
- Card sorting or grouping
- Animations
- Mana cost symbols
- Playability hints

## Dependencies

- Phase 1c complete (battlefield display with PixiJS)

---

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-17
**Completed**: 2026-01-17
**Agent**: ui-engineer

### Task Breakdown

#### Phase 1: Hand Zone Component (PixiJS) ✅
- [x] Create `src/app/games/[gameId]/components/hand/HandZone.tsx`
- [x] Implement horizontal layout with 60px overlap (120px visible per card)
- [x] Use existing CardSprite component (rotation=0, cards always upright)
- [x] Center row horizontally at Y=925
- [x] Test with 0, 1, and 7 cards

#### Phase 2: Opponent Hand Count (HTML) ✅
- [x] Create `src/app/games/[gameId]/components/hand/OpponentHandCount.tsx`
- [x] Display "Opponent's Hand: X card(s)" with singular/plural
- [x] Style: background #0D1117, text #B0B0B0, Inter 16px semi-bold

#### Phase 3: Integration ✅
- [x] Extend BattlefieldDisplay canvas to include hand zone (Y 820-1040)
- [x] Add separator zone at Y 780-820 (#0D1117)
- [x] Add HandZone component below battlefield
- [x] Add OpponentHandCount above canvas
- [x] Pass correct props from GameSnapshot

#### Phase 4: Testing and QA ✅
- [x] Test empty hand (no crash)
- [x] Test singular/plural: "1 card" vs "X cards"
- [x] Verify visual style matches design spec
- [x] Run `bun run lint && bun run format`
- [x] Run `bun test`
- [x] Manual visual verification (build succeeds)

**Blockers**: None

**Notes**:
- Heavy reuse of Phase 1c components (CardSprite, CardTextureGenerator, CardTextureCache)
- Hand cards use 60px overlap (120px visible) vs battlefield 20px spacing
- Opponent hand count is HTML/CSS (not PixiJS) for simplicity
- Card dimensions: 180x250px (same as battlefield)
- Canvas height extended from 1080 to 1220 to accommodate hand zone
- All components follow existing patterns from battlefield display
- Empty hand handled gracefully (returns null, no crash)
