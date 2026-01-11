# UI MVP Specification

This document specifies the minimal playable UI for Echomancy from a product perspective.

---

## 1. Overview

### Purpose

Enable two players to play a complete game of Echomancy on a single screen, taking turns.

### Design Goals

1. **Clarity**: Game state is always unambiguous to players
2. **Simplicity**: Minimal visual complexity - function over form
3. **Iterative**: Built in tiny increments, each one testable

### Scope

**In Scope:**
- See cards in hand, battlefield, graveyard
- See the stack of spells/abilities waiting to resolve
- Know whose turn it is and who has priority
- Play lands, cast spells, activate abilities
- Declare attackers and blockers
- Pass priority and end turn

**Out of Scope:**
- Deck builder
- Matchmaking / online play
- Replays
- Animations
- Sound
- Mobile layouts

---

## 2. User Stories

### As a player, I want to...

1. **See my hand** so I can decide what to play
2. **See the battlefield** so I can understand the board state
3. **See life totals** so I know who's winning
4. **See whose turn it is** so I know when I can act
5. **Play a land** during my main phase
6. **Cast a spell** and select targets if needed
7. **Activate an ability** on my permanents
8. **Attack with my creatures** during combat
9. **Block with my creatures** when attacked
10. **Pass priority** to let the opponent respond or resolve spells
11. **End my turn** when I'm done

---

## 3. Player Experience

### What the player sees

**Game Layout (top to bottom):**
- Opponent's area: their battlefield, life total, hand size (cards hidden)
- Middle area: stack of spells, turn/phase indicator, priority indicator
- Player's area: their battlefield, hand (cards visible), life total, mana pool

### Core Interactions

| I want to... | I do this... | I see this... |
|--------------|--------------|---------------|
| Play a land | Click land in hand | Land moves to battlefield |
| Cast a spell | Click spell in hand, select targets if needed, confirm | Spell appears on stack |
| Activate ability | Click permanent, select targets if needed, confirm | Ability appears on stack |
| Attack | Click creatures to attack, confirm | Creatures marked as attacking |
| Block | Click my creature, click attacker to block, confirm | Blocking relationship shown |
| Pass priority | Click "Pass" button | Priority passes to opponent |
| End turn | Click "End Turn" button | Turn advances to opponent |

### Priority Flow

When I have priority:
- "Pass" button is enabled
- I can play cards and activate abilities
- My area is highlighted

When opponent has priority:
- "Pass" button is disabled
- I wait for them to act
- Opponent's area is highlighted

---

## 4. Game Rules & Mechanics

### Turn Structure Display

Player should always know:
- Current turn number
- Current phase (Beginning, Main, Combat, End)
- Current step within combat (Declare Attackers, Declare Blockers, Damage)
- Who has priority

### Stack Visibility

- All spells and abilities on the stack are visible to both players
- Top of stack (resolves first) is clearly indicated
- Each item shows: what it is, who controls it, what it targets

### Combat Flow

1. Active player enters combat
2. Active player selects attackers (click to toggle)
3. Active player confirms attackers
4. Defending player selects blockers (click blocker, then click attacker)
5. Defending player confirms blockers
6. Damage happens automatically

### Targeting

When a spell or ability needs targets:
1. Player clicks the card/ability
2. Valid targets are highlighted
3. Player clicks targets to select
4. Player confirms or cancels

---

## 5. Phases (Tiny Steps)

### Phase 0: Debug Console

**Goal**: Validate engine integration without building real UI.

**What it is**: A simple textarea where you can paste JSON actions and see the resulting game state.

**Player experience**:
1. Page shows a textarea and a "Submit" button
2. Paste a JSON action (e.g., `{"type": "PLAY_LAND", "playerId": "p1", "cardId": "forest-1"}`)
3. Click Submit
4. See the raw game state JSON below
5. See any errors if the action was invalid

**Acceptance criteria**:
- Can create a new game
- Can submit any valid action as JSON
- Can see the full game state after each action
- Shows clear error messages for invalid actions

**Why this matters**: Proves the engine works before investing in UI. Enables rapid testing of edge cases.

### Phase 1: Read-Only Display

**Goal**: See the game state visually.

**Player experience**:
- See cards in hand (names and basic info)
- See cards on battlefield (both players)
- See life totals
- See current turn/phase
- See stack contents
- NO interactions yet

**Acceptance criteria**:
- All zones display their cards
- Tapped cards look different from untapped
- Creatures show power/toughness
- Turn and phase are visible

### Phase 2: Basic Actions

**Goal**: Play lands, cast simple spells, pass priority.

**Player experience**:
- Click land in hand → plays land
- Click "Pass Priority" → passes
- Click "End Turn" → ends turn
- See stack update when spells are cast
- See errors when illegal actions attempted

**Acceptance criteria**:
- Can play one land per turn during main phase
- Cannot play land at wrong time (shows error)
- Can pass priority
- Can end turn

### Phase 3: Spell Casting with Targets

**Goal**: Cast spells that require targets.

**Player experience**:
- Click spell in hand
- Valid targets highlight
- Click target to select
- Confirm or cancel
- Spell goes on stack with target

**Acceptance criteria**:
- Targeted spells prompt for target selection
- Only valid targets can be selected
- Can cancel mid-selection
- Spell resolves correctly on target

### Phase 4: Combat

**Goal**: Full combat flow.

**Player experience**:
- During Declare Attackers: click creatures to toggle attack
- Confirm attackers
- During Declare Blockers: click creature, then click attacker
- Confirm blockers
- Damage resolves automatically
- Dead creatures go to graveyard

**Acceptance criteria**:
- Can declare multiple attackers
- Can declare blockers
- Flying can only be blocked by Flying/Reach
- Combat damage updates life totals
- Creatures die when damage >= toughness

### Phase 5: Polish

**Goal**: Complete, playable MVP.

**Player experience**:
- Graveyard can be expanded to see all cards
- Activated abilities work (mana abilities, etc.)
- All error states handled gracefully
- Clear visual feedback for all actions

**Acceptance criteria**:
- Can play a complete game from start to finish
- Two players can take turns on the same screen
- All edge cases from engine tests work in UI

---

## 6. Out of Scope

### Explicitly NOT included in MVP

- Deck building
- Online multiplayer
- Player accounts
- Match history
- Card art/images (text only is fine)
- Animations or transitions
- Sound effects
- Mobile responsive design
- Undo functionality
- Timer/clock

### Future Considerations

- WebSocket for real-time multiplayer
- Drag and drop for targeting
- Card preview on hover
- Keyboard shortcuts

---

## 7. Acceptance Criteria (MVP Complete)

The UI MVP is complete when:

1. Two players can play a full game on one screen
2. All card types work (creatures, lands, spells)
3. Combat resolves correctly
4. Stack resolves in correct order
5. Priority passes correctly between players
6. All existing engine tests can be reproduced through the UI
7. Error messages are clear and helpful

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-29 | Initial specification |
| 2.0 | 2025-12-29 | Removed implementation details, added Phase 0, product focus |
