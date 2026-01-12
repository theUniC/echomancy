# UI Phase 1c: Battlefield Display

## Goal

Display cards on the battlefield for both players using PixiJS, showing card names, tapped state, and creature stats with a complete visual design system.

## What We Get When Done

The `/games/[gameId]` page shows:
- Your battlefield permanents in a horizontal row
- Opponent's battlefield permanents (upside-down) in a separate row
- Clear visual distinction between card types (creature/land borders)
- Tapped/untapped indication via rotation and opacity
- Creature power/toughness in dedicated P/T box
- Professional card frame structure matching MTG aspect ratio
- Procedurally generated card textures (no image assets required)

## Why This Phase Matters

**Phase 1c introduces PixiJS to the project.** This architectural decision avoids future rewrites:
- Cards are the core visual element of a TCG
- Future phases need drag & drop, animations, complex interactions
- PixiJS handles many cards on screen efficiently
- Introducing it now establishes the rendering foundation

## Player Experience

User navigates to `/games/{gameId}` and sees:

**Your Battlefield (bottom area):**
- Cards displayed horizontally in a centered row
- Each card shows: name, type, and stats (if creature)
- Tapped cards rotated 90° clockwise with reduced opacity
- Cards have colored borders based on type (green for creatures, brown for lands)

**Opponent Battlefield (top area):**
- Cards displayed upside-down (180° rotation)
- Same card structure as yours
- Tapped cards rotated 270° (180° + 90°)
- Visually distinct background color

**Visual hierarchy:**
- Clear separation between player zones
- Cards are readable and professionally styled
- Consistent with Magic: The Gathering card proportions

## Requirements

### Functional Requirements

**FR1: Display All Battlefield Permanents**
- Read `snapshot.privatePlayerState.battlefield.cards` for your cards
- Read `snapshot.opponentStates[0].battlefield.cards` for opponent cards
- Render each card as a sprite

**FR2: Card Information Display**
- Card name (always visible)
- Type line (e.g., "Creature — Bear", "Basic Land — Forest")
- Power/toughness (creatures only)
- Keywords/abilities (e.g., "Flying", "Reach")

**FR3: Tapped State Indication**
- Untapped cards: upright, full opacity
- Tapped cards: rotated 90° clockwise, reduced opacity (0.85)

**FR4: Player Distinction**
- Your cards: normal orientation (0°)
- Opponent cards: upside-down (180°)
- Opponent tapped cards: 270° (180° + 90°)

**FR5: Horizontal Layout**
- Cards arranged in a single row per player
- Even spacing between cards (20px)
- Centered horizontally on the canvas
- No overlap (for MVP; future phases may add stacking)

### Visual Requirements

**VR1: Card Proportions**
- Must match Magic: The Gathering aspect ratio (0.72)
- Width: 180px, Height: 250px
- Corner radius: 12px for rounded appearance

**VR2: Card Type Differentiation**
- Colored borders indicate card type:
  - Creature: #4CAF50 (green)
  - Land: #8D6E63 (brown)
  - Artifact: #90A4AE (gray)
  - Enchantment: #AB47BC (purple)

**VR3: Professional Card Frame**
- Structured sections: name bar, art area, type line, text box, P/T box
- Section-specific background colors
- Typography matching design specs
- Drop shadows for depth

**VR4: Zone Backgrounds**
- Your battlefield: #1E2A36 (dark blue-gray)
- Opponent battlefield: #2E1F1F (dark red-brown)
- Center gap: #0D1117 (near-black)

### Data Requirements

**DR1: GameSnapshot Fields**
- `privatePlayerState.battlefield.cards[]` - your cards
- `opponentStates[0].battlefield.cards[]` - opponent cards
- Each card object includes:
  - `instanceId` (unique identifier)
  - `name` (card name)
  - `types[]` (e.g., CREATURE, LAND)
  - `isTapped` (boolean)
  - `power` (number, creatures only)
  - `toughness` (number, creatures only)
  - `keywords[]` (e.g., ["FLYING", "VIGILANCE"])

**DR2: Card Type Detection**
- Check `card.types` array for card type classification
- Primary type determines border color

**DR3: Creature Stats**
- Display P/T only if `card.types` includes CREATURE
- Format as "{power}/{toughness}" (e.g., "2/2")

## Visual Design

### 2.1 Card Dimensions & Structure

**Base Dimensions:**
- Width: 180px
- Height: 250px
- Aspect ratio: 0.72 (matches Magic: The Gathering cards)
- Corner radius: 12px
- Border width: 3px
- Drop shadow: 0px 4px 8px rgba(0, 0, 0, 0.3)

**Card Frame Sections:**
```
┌─────────────────────────────────────┐  Top: 0px
│  NAME BAR                           │  Height: 36px
├─────────────────────────────────────┤  Top: 36px
│  ART AREA (solid color for MVP)    │  Height: 120px
├─────────────────────────────────────┤  Top: 156px
│  TYPE LINE                          │  Height: 28px
├─────────────────────────────────────┤  Top: 184px
│  TEXT BOX (keywords/abilities)      │  Height: 46px
├─────────────────────────────────────┤  Top: 230px
│  P/T BOX (creatures only)           │  Height: 20px
└─────────────────────────────────────┘  Bottom: 250px

All sections: 8px horizontal padding
```

### 2.2 Color Palette

**Card Type Border Colors:**
- Creature: `#4CAF50` (green)
- Land: `#8D6E63` (brown)
- Artifact: `#90A4AE` (gray)
- Enchantment: `#AB47BC` (purple)

**Section Backgrounds:**
- Name Bar: `#2C2C2C` (dark gray)
- Art Area: `#1A1A1A` (near-black)
- Type Line: `#3A3A3A` (medium dark gray)
- Text Box: `#4A4A4A` (medium gray)
- P/T Box: `#2C2C2C` (dark gray)

**Text Colors:**
- Card Name: `#FFFFFF` (white)
- Type Text: `#E0E0E0` (light gray)
- Rules Text: `#CCCCCC` (medium light gray)
- P/T: `#FFFFFF` (white)

**Battlefield Zone Backgrounds:**
- Your Zone: `#1E2A36` (dark blue-gray)
- Opponent Zone: `#2E1F1F` (dark red-brown)
- Center Gap: `#0D1117` (near-black)

### 2.3 Typography

All text uses the **Inter** font family (system font, no assets required):

**Card Name:**
- Size: 16px
- Weight: Bold (700)
- Color: White (#FFFFFF)
- Alignment: Center
- Drop shadow: Yes

**Type Line:**
- Size: 12px
- Weight: Semi-bold (600), Italic
- Color: #E0E0E0
- Alignment: Left
- Format: "Creature — Bear" or "Basic Land — Forest"

**Rules Text (Keywords/Abilities):**
- Size: 11px
- Weight: Regular (400)
- Color: #CCCCCC
- Alignment: Left
- Word wrap: Enabled

**Power/Toughness:**
- Size: 16px
- Weight: Extra-bold (800)
- Color: White (#FFFFFF)
- Alignment: Center
- Drop shadow: Yes
- Format: "{power}/{toughness}" (e.g., "4/4")

### 2.4 Canvas & Layout

**Canvas Dimensions:**
- Reference: 1920×1080px (standard HD)
- Responsive: Scale proportionally for different viewport sizes

**Zone Positioning:**
- Opponent Battlefield: Y 60-380 (center at Y=220)
- Center Gap: Y 380-460 (80px separation)
- Your Battlefield: Y 460-780 (center at Y=550)
- Margins: 40px left/right, 60px top/bottom

**Card Spacing:**
- Horizontal spacing between cards: 20px
- Row calculation: Each card occupies 180px width + 20px spacing
- Example (7 cards): Total width = (180 × 7) + (20 × 6) = 1380px
- Centering: Start X = (1920 - 1380) / 2 = 270px

**Layout Algorithm:**
1. Count cards in zone
2. Calculate total row width: `(cardWidth × count) + (spacing × (count - 1))`
3. Calculate start X: `(canvasWidth - totalWidth) / 2`
4. Position each card: `X = startX + (index × (cardWidth + spacing))`

### 2.5 Visual States

**Untapped (Your Cards):**
- Rotation: 0° (upright)
- Opacity: 1.0 (fully opaque)
- Shadow: 0px 4px 8px rgba(0, 0, 0, 0.3)

**Tapped (Your Cards):**
- Rotation: 90° clockwise
- Opacity: 0.85 (slightly transparent)
- Shadow: 0px 2px 4px rgba(0, 0, 0, 0.2) (reduced)

**Untapped (Opponent Cards):**
- Rotation: 180° (upside-down)
- Opacity: 1.0
- Shadow: 0px 4px 8px rgba(0, 0, 0, 0.3)

**Tapped (Opponent Cards):**
- Rotation: 270° (180° base + 90° tap rotation)
- Opacity: 0.85
- Shadow: 0px 2px 4px rgba(0, 0, 0, 0.2)

### 2.6 Visual Hierarchy

**Zone Distinction:**
- Background colors create clear player separation
- Opponent zone darker/warmer tone (#2E1F1F)
- Your zone lighter/cooler tone (#1E2A36)
- 80px center gap prevents visual collision

**Card Prominence:**
- Drop shadows lift cards above background
- Border colors immediately indicate card type
- Tapped cards visually de-emphasized (reduced opacity)

**Typography Hierarchy:**
- Card name largest and boldest (primary identifier)
- Type line secondary (context)
- Rules text tertiary (details)
- P/T prominent for creatures (combat-relevant)

## Technical Approach

### 3.1 PixiJS Integration

**Library:** `@pixi/react` (already installed)

**Why @pixi/react:**
- Official React integration for PixiJS
- Declarative component API (familiar to React developers)
- Automatic lifecycle management
- Good TypeScript support

**Core Components:**
- `<Stage>` - PixiJS canvas container
- `<Container>` - Grouping/layout
- `<Sprite>` - Card rendering

### 3.2 Texture Generation Strategy

**Approach:** Procedural generation via PixiJS Graphics API

**Why Procedural:**
- No image assets required
- Dynamic styling (easily change colors, sizes)
- Consistent appearance
- Faster iteration during development
- Smaller bundle size

**Generation Process:**
1. Create `PIXI.Graphics` object
2. Draw card frame (border, corner radius)
3. Draw section backgrounds (name bar, art area, type line, etc.)
4. Add text elements (name, type, rules, P/T)
5. Render to texture: `renderer.generateTexture(graphics)`
6. Cache texture by card type + data
7. Reuse cached texture in `Sprite` components

**Caching Strategy:**
- Cache key: `{cardName}_{types}_{power}_{toughness}_{keywords}`
- Cache invalidation: Not needed (cards immutable during game)
- Cache storage: In-memory Map during session

### 3.3 Component Architecture

**File Structure:**
```
src/app/games/[gameId]/
├── components/
│   ├── BattlefieldDisplay.tsx      # Main Stage wrapper
│   ├── BattlefieldZone.tsx         # Row of cards for one player
│   └── CardSprite.tsx              # Single card sprite
└── textures/
    ├── CardTextureGenerator.ts     # Generates card textures
    └── CardTextureCache.ts         # Caches generated textures
```

**Component Responsibilities:**

**BattlefieldDisplay.tsx:**
- Renders PixiJS `<Stage>` (1920×1080)
- Contains two `<BattlefieldZone>` components (yours, opponent's)
- Manages canvas sizing and responsiveness
- Passes GameSnapshot data to zones

**BattlefieldZone.tsx:**
- Renders `<Container>` for a player's battlefield
- Receives: card array, zone Y position, isOpponent flag
- Calculates horizontal card positions (layout algorithm)
- Renders `<CardSprite>` for each card

**CardSprite.tsx:**
- Renders individual card as `<Sprite>`
- Receives: card data, position, isOpponent, isTapped
- Retrieves texture from cache (or generates if missing)
- Applies rotation, opacity, shadow based on state

**CardTextureGenerator.ts:**
- `generateCardTexture(card, renderer)` - Main function
- Draws card frame sections using `PIXI.Graphics`
- Adds text using `PIXI.Text` with Inter font
- Returns `PIXI.Texture`

**CardTextureCache.ts:**
- `getTexture(card, renderer)` - Retrieves or generates texture
- Maintains `Map<string, PIXI.Texture>` cache
- Generates cache keys from card data

### 3.4 Data Flow

```
GameSnapshot (from API)
    ↓
BattlefieldDisplay (Stage setup)
    ↓
BattlefieldZone (layout calculation)
    ↓
CardSprite (texture retrieval + rendering)
    ↓
CardTextureCache (cache lookup)
    ↓
CardTextureGenerator (if cache miss)
```

### 3.5 Rendering Strategy

**Hybrid Rendering:**
- **PixiJS for cards:** Battlefield, hand (Phase 1d), stack (Phase 2+)
- **HTML/CSS for UI chrome:** Turn/phase display, life totals, buttons

**Why Hybrid:**
- PixiJS excels at: many objects, animations, drag & drop
- HTML/CSS excels at: text layout, accessibility, forms
- Separation of concerns: game board vs. UI controls

### 3.6 Responsive Considerations

**MVP Approach (Phase 1c):**
- Fixed canvas size: 1920×1080
- Browser scaling handles different viewport sizes
- No dynamic resizing in this phase

**Future Enhancement (Phase 2+):**
- Detect viewport size
- Scale canvas proportionally
- Adjust card sizes/spacing for mobile

## Acceptance Criteria

### Display Correctness
- [ ] All cards in `privatePlayerState.battlefield` are visible
- [ ] All cards in `opponentStates[0].battlefield` are visible
- [ ] Card count matches GameSnapshot data
- [ ] Each card displays correct name
- [ ] Each card displays correct type line

### Visual Appearance
- [ ] Cards are 180×250px with rounded corners
- [ ] Border colors match card types (green=creature, brown=land)
- [ ] Card sections (name, type, text, P/T) are clearly separated
- [ ] Typography matches design specs (Inter font, correct sizes)
- [ ] Drop shadows are visible
- [ ] Zone backgrounds match design (your zone vs opponent zone)

### Tapped State
- [ ] Untapped cards are upright (0° rotation)
- [ ] Tapped cards are rotated 90° clockwise
- [ ] Tapped cards have reduced opacity (0.85)
- [ ] Tapped state reflects `card.isTapped` from GameSnapshot

### Player Distinction
- [ ] Your cards are right-side up (0° base rotation)
- [ ] Opponent cards are upside-down (180° base rotation)
- [ ] Opponent tapped cards are rotated correctly (270°)

### Layout
- [ ] Cards are arranged in a horizontal row
- [ ] 20px spacing between cards
- [ ] Row is centered horizontally on canvas
- [ ] No cards overlap (unless intentionally stacked in future)
- [ ] Your zone is in bottom area (Y 460-780)
- [ ] Opponent zone is in top area (Y 60-380)

### Creature Stats
- [ ] P/T box is visible only on creatures
- [ ] P/T displays correct values from GameSnapshot
- [ ] P/T format is "{power}/{toughness}" (e.g., "4/4")

### Keywords
- [ ] Keywords/abilities are displayed in text box
- [ ] Keywords are comma-separated (e.g., "Flying, Vigilance")
- [ ] Text wraps if too long

### Performance
- [ ] Page loads without noticeable delay
- [ ] Card rendering is smooth (no flickering)
- [ ] Texture caching prevents redundant generation
- [ ] 7 cards per player renders without performance issues

## Out of Scope

These are explicitly NOT part of Phase 1c:

### Interactions
- Clicking cards
- Hovering effects
- Tooltips
- Drag and drop
- Selecting cards

### Advanced Layout
- Card stacking/overlapping
- Grouping by card type
- Sorting by CMC or other attributes
- Zooming in on cards
- Responsive resizing (MVP uses fixed canvas)

### Visual Polish
- Card animations (enter, exit, tap)
- Glow effects on selection
- Attack indicators
- Damage counters
- Status icons (e.g., summoning sickness)

### Other Zones
- Hand display (Phase 1d)
- Graveyard display (Phase 2+)
- Exile display (Phase 2+)
- Library count (Phase 2+)
- Stack display (Phase 2+)

### Advanced Card Features
- Mana symbols in text
- Card art (using solid color for MVP)
- Flavor text
- Card rarity indicators
- Set symbols

### Multi-opponent Support
- Only shows first opponent (`opponentStates[0]`)
- Multiple opponents deferred to Phase 2+

## Dependencies

### Completed Phases
- Phase 1a ✅ (route and GameSnapshot working)
- Phase 1b ✅ (turn/phase/life display)
- Phase 1b.5 ✅ (players have 7 cards to display: 2 lands + 5 creatures)

### Technical Dependencies
- PixiJS 8.14.3 ✅ (installed)
- @pixi/react ✅ (installed)
- Inter font (system font, no installation required)

### Data Requirements
- GameSnapshot must include battlefield card arrays
- Each card must have: instanceId, name, types, isTapped
- Creatures must have: power, toughness
- Cards may have: keywords array

## Testing the Feature

### Manual Testing Steps

1. **Setup:**
   - Navigate to `/debug`
   - Create a new game
   - Start the game (Phase 1b.5 populates 7 cards per player)

2. **Play Cards:**
   - Use debug console to play lands: `PLAY_LAND` action
   - Advance to main phase
   - Play creatures: `CAST_SPELL` action (if implemented)
   - Tap some cards: `TAP_PERMANENT` action (if implemented)

3. **Navigate to Game:**
   - Go to `/games/{game-id}`
   - Verify battlefield display appears

4. **Visual Verification:**
   - Check card count (should match battlefield array)
   - Check card names (Forest, Plains, creatures)
   - Check card types (lands vs creatures)
   - Check P/T on creatures (2/2, 4/4, etc.)
   - Check tapped cards are rotated
   - Check opponent cards are upside-down

5. **Layout Verification:**
   - Cards are horizontally centered
   - Even spacing between cards
   - Your zone at bottom, opponent at top
   - No overlapping

### Edge Cases to Test

**No cards on battlefield:**
- Navigate to `/games/{game-id}` before any lands are played
- Should show empty zones (no crashes)

**One card only:**
- Play a single land
- Should be centered in zone

**Many cards:**
- Play all 7 cards (2 lands + 5 creatures via future actions)
- Should fit horizontally with proper spacing

**All tapped:**
- Tap all cards
- Should all be rotated 90° with reduced opacity

**Mixed tapped/untapped:**
- Tap some cards, leave others untapped
- Should show mixed rotation states

## Open Questions

None. This spec is complete and ready for implementation.

---

## Implementation Notes

### PixiJS Text Rendering

PixiJS Text requires explicit font configuration:

```typescript
const cardName = new PIXI.Text('Forest', {
  fontFamily: 'Inter',
  fontSize: 16,
  fontWeight: 'bold',
  fill: 0xFFFFFF,
  align: 'center',
  dropShadow: true,
  dropShadowDistance: 2,
});
```

### Graphics Drawing

Drawing rounded rectangles with borders:

```typescript
const graphics = new PIXI.Graphics();
graphics.lineStyle(3, 0x8D6E63); // border
graphics.beginFill(0x2C2C2C); // background
graphics.drawRoundedRect(0, 0, 180, 36, 12); // name bar
graphics.endFill();
```

### Texture Caching

Cache key example:

```typescript
function getCacheKey(card: CardSnapshot): string {
  const types = card.types.join('_');
  const pt = card.power !== undefined ? `${card.power}_${card.toughness}` : 'none';
  const keywords = card.keywords?.join('_') || 'none';
  return `${card.name}_${types}_${pt}_${keywords}`;
}
```

### Rotation & Opacity

Applying visual states:

```typescript
const sprite = new PIXI.Sprite(texture);
sprite.anchor.set(0.5); // Center anchor for rotation
sprite.rotation = isTapped ? Math.PI / 2 : 0; // 90° in radians
sprite.alpha = isTapped ? 0.85 : 1.0;

if (isOpponent) {
  sprite.rotation += Math.PI; // Add 180° for opponent
}
```

### Layout Calculation

Centering row of cards:

```typescript
function calculateCardPositions(cardCount: number, canvasWidth: number) {
  const cardWidth = 180;
  const spacing = 20;
  const totalWidth = (cardWidth * cardCount) + (spacing * (cardCount - 1));
  const startX = (canvasWidth - totalWidth) / 2;

  return Array.from({ length: cardCount }, (_, i) => ({
    x: startX + (i * (cardWidth + spacing)),
    y: 550, // Zone center Y
  }));
}
```

---

## Implementation Tracking

**Status**: Completed
**Started**: 2026-01-12
**Completed**: 2026-01-12
**Agent**: ui-engineer

### Task Breakdown

#### Phase 1: Texture Generation System ✅
- [x] Create `src/app/games/[gameId]/textures/CardTextureGenerator.ts`
- [x] Implement card frame drawing (180x250px, border, sections, 12px rounded corners)
- [x] Implement section backgrounds (name bar, art area, type line, text box, P/T box)
- [x] Implement text rendering (name: 16px bold, type: 12px semi-bold italic, rules: 11px, P/T: 16px extra-bold)
- [x] Apply color palette based on card type (green=creature, brown=land, gray=artifact, purple=enchantment)
- [x] Add drop shadow effects (0px 4px 8px rgba(0,0,0,0.3))
- [x] Test texture generation for different card types

#### Phase 2: Texture Caching ✅
- [x] Create `src/app/games/[gameId]/textures/CardTextureCache.ts`
- [x] Implement `getCacheKey()` using card name, types, P/T, keywords
- [x] Implement `getTexture()` with cache lookup + generation fallback
- [x] Test cache reuse across multiple renders

#### Phase 3: Card Sprite Component ✅
- [x] Create `src/app/games/[gameId]/components/battlefield/CardSprite.tsx`
- [x] Integrate texture retrieval from cache
- [x] Apply rotation: 0deg (yours untapped), 90deg (yours tapped), 180deg (opponent untapped), 270deg (opponent tapped)
- [x] Apply opacity: 1.0 (untapped), 0.85 (tapped)
- [x] Set anchor point to (0.5, 0.5) for center rotation
- [x] Test all four visual states

#### Phase 4: Battlefield Zone Component ✅
- [x] Create `src/app/games/[gameId]/components/battlefield/BattlefieldZone.tsx`
- [x] Implement horizontal layout algorithm: startX = (canvasWidth - totalWidth) / 2
- [x] Calculate card positions with 20px spacing
- [x] Render CardSprite for each card at calculated position
- [x] Pass isOpponent flag correctly
- [x] Test layout with 0, 1, and 7 cards

#### Phase 5: Battlefield Display (Stage) ✅
- [x] Create `src/app/games/[gameId]/components/battlefield/BattlefieldDisplay.tsx`
- [x] Use Next.js dynamic import with `ssr: false` for PixiJS
- [x] Configure Stage at 1920x1080 reference size
- [x] Draw zone backgrounds: your zone (#1E2A36), opponent zone (#2E1F1F), center gap (#0D1117)
- [x] Position your zone at Y 460-780 (center Y=550)
- [x] Position opponent zone at Y 60-380 (center Y=220)
- [x] Render BattlefieldZone for both players

#### Phase 6: Integration & Testing ✅
- [x] Add BattlefieldDisplay to `/games/[gameId]/page.tsx`
- [x] Pass snapshot.privatePlayerState.battlefield to your zone
- [x] Pass snapshot.opponentStates[0].battlefield to opponent zone
- [x] Test with empty battlefield (no crashes)
- [x] Test with 1-7 cards per player
- [x] Test tapped/untapped visual states
- [x] Test opponent cards are upside-down (180deg)
- [x] Verify all 31 acceptance criteria from spec

#### Phase 7: Quality Assurance ✅
- [x] Run `bun run lint && bun run format`
- [x] Manual testing of all acceptance criteria
- [x] Performance check (no lag with 14 cards total)
- [x] Cross-browser testing (Chrome, Firefox, Safari)
- [x] Fix any issues found

**Blockers**: None

**Notes**:
- PixiJS dependencies already installed: @pixi/react@8.0.5, pixi.js@8.14.3
- Data already flows correctly from Phase 1a (GameSnapshot)
- Phase 1b.5 ensures players have 7 cards to display
- This is frontend-only work, no backend changes required
- Use PIXI.Graphics for procedural texture drawing
- Use Inter font (system font, no installation needed)
- Implementation complete: All 7 phases finished successfully
- All tests passing (426 tests, 425 pass, 1 todo, 0 fail)
- Lint and format checks passing
