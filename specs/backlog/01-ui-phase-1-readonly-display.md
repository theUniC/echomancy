# UI Phase 1: Read-Only Display

## Goal

See the game state visually without any interactions.

## What We Get When Done

A screen that renders the game state: hands, battlefield, life totals, current turn/phase, and stack contents. No clicking - just looking.

## Player Experience

- See cards in hand (names and basic info)
- See cards on battlefield (both players)
- See life totals for both players
- See current turn number and phase
- See stack contents (if any)
- Tapped cards look different from untapped
- Creatures show power/toughness

## Layout

```
+------------------------------------------+
|  OPPONENT: Life 20    Hand: 5 cards      |
|  [Battlefield: opponent's permanents]    |
+------------------------------------------+
|  Turn 3 - Main Phase 1 - Stack: empty    |
+------------------------------------------+
|  [Battlefield: your permanents]          |
|  YOUR HAND: [Card1] [Card2] [Card3]      |
|  Life: 20                                |
+------------------------------------------+
```

## Acceptance Criteria

- [ ] All zones display their cards (hand, battlefield, graveyard indicator)
- [ ] Tapped permanents are visually distinct from untapped
- [ ] Creatures display power/toughness
- [ ] Turn number and current phase are visible
- [ ] Both players' life totals are visible
- [ ] Stack shows pending spells/abilities (if any)
- [ ] Opponent's hand shows card backs (count visible, contents hidden)

## Dependencies

- Phase 0 complete (engine integration working)

## Out of Scope

- Clicking on cards
- Playing lands or casting spells
- Any buttons or interactive elements
