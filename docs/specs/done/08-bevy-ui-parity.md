# Bevy UI: TypeScript Parity

## Goal

Port the existing TypeScript/React UI to Bevy, achieving exact feature parity. Nothing more, nothing less.

## Scope

This spec covers ONLY the features that exist in the completed TypeScript UI (specs 00 through 07). It does NOT include any backlog items (08-16: spell casting, combat UI, etc.).

## What the TypeScript UI Does (EXHAUSTIVE)

### Visual Elements
1. Game window
2. Player battlefield -- horizontal row of cards (creatures/lands)
3. Opponent battlefield -- horizontal row, visually distinct from player's
4. Player hand -- horizontal row with overlapping cards (60px overlap in TS)
5. Tapped cards rotated 90 degrees, slightly transparent (alpha 0.85)
6. Procedural card rendering: colored border by type (green=creature, brown=land), name, type line, P/T, rules text
7. Green border on playable cards in hand
8. Priority indicator ("Your Priority" in green, "Opponent's Priority" in gray)
9. "Pass" and "End Turn" buttons (disabled when not your priority)
10. Turn display: "Turn {N} - {phase} {step}"
11. Life totals for both players
12. Opponent hand count
13. Graveyard counts for both players
14. Error messages (red, dismissable)

### Interactions
1. Click a playable card in hand -> plays it as a land
2. Click "Pass" button -> advance step
3. Click "End Turn" button -> end turn

### Data Flow
- Game state comes from `Game` struct (domain model)
- `GameSnapshot` provides player-relative view via `create_game_snapshot()`
- `AllowedActionsResult` tells which cards are playable
- Action -> `game.apply()` -> re-derive snapshot -> update visuals

## Architecture

### Bevy Integration Pattern

```
echomancy-core (lib)          echomancy-bevy (bin)
-----------------------       --------------------------------
Game                    --->  Resource<GameState>
GameStateExport         --->  derived each frame via system
GameSnapshot            --->  drives all UI rendering
AllowedActionsResult    --->  drives playable card highlights
Action enum             <---  produced by input systems
```

### Plugin Structure

```rust
EchomancyPlugin
  +-- GamePlugin         (Resource<GameState>, events, state sync)
  +-- UiPlugin           (camera, root layout)
      +-- HudPlugin      (turn info, life totals, priority, buttons, counts)
      +-- BattlefieldPlugin (player + opponent battlefield rendering)
      +-- HandPlugin     (hand display, card click interaction)
      +-- ErrorPlugin    (error message display + dismiss)
```

### Card Rendering Approach

Use Bevy UI nodes (not sprites, not procedural textures):
- Each card is a `Node` with background color based on type
- Text children for name, type line, P/T
- Border color for playable indication
- Transform rotation for tapped state
- This matches the TypeScript approach (DOM nodes with CSS) and is simplest in Bevy

### State Synchronization

- `GameSnapshot` is recomputed whenever game state changes (not every frame)
- A `Changed<GameState>` system trigger rebuilds the snapshot
- UI systems read the snapshot resource to render

### Game Initialization

On startup:
1. Create Game, add two players, assign decks, start game
2. Store as `Resource<GameState>` wrapping the `Game`
3. Player 1 is the "viewer" (local player)
4. Auto-advance through untap/upkeep/draw to first main phase

## Acceptance Criteria

- [ ] Window opens with "Echomancy" title
- [ ] Game initializes with two players and bootstrap hands
- [ ] Player's battlefield shows cards in horizontal row
- [ ] Opponent's battlefield shows cards in horizontal row (visually distinct)
- [ ] Player's hand shows 7 cards with overlap
- [ ] Cards show: colored border (green=creature, brown=land), name, type line, P/T
- [ ] Tapped cards appear rotated 90 degrees with reduced opacity
- [ ] Playable lands in hand have green border highlight
- [ ] Turn display shows "Turn {N} - {phase} {step}"
- [ ] Life totals shown for both players
- [ ] Opponent hand count displayed
- [ ] Graveyard counts displayed for both players
- [ ] Priority indicator shows who has priority with color coding
- [ ] "Pass" button advances step when clicked (disabled without priority)
- [ ] "End Turn" button ends turn when clicked (disabled without priority)
- [ ] Clicking a playable land in hand plays it to battlefield
- [ ] Error messages appear in red and can be dismissed
- [ ] UI updates reactively when game state changes

## Out of Scope

- Combat UI (declare attackers/blockers/damage) -- backlog spec 09-11
- Spell casting -- backlog spec 08
- Activated abilities -- backlog spec 13
- Stack display -- backlog spec 15
- Graveyard viewer (detailed) -- backlog spec 12
- Exile zone -- backlog spec 16
- Game end display -- backlog spec 14
- AI opponent / networking
- Sound effects or animations
- Card art / images

## Implementation Tracking

**Status**: In Progress
**Started**: 2026-03-27
**Completed**:
**Agent**: senior-backend-engineer

### Task Breakdown

#### Sub-phase 1: Plugin Skeleton + Game Resource + Camera (Small) - DONE

**Objective**: Establish the Bevy plugin architecture and bridge the domain Game into the ECS.

- [x] Create `EchomancyPlugin` in `crates/echomancy-bevy/src/plugins/mod.rs`
- [x] Create `GamePlugin` that initializes a 2-player game as a `Resource`
- [x] Implement `GameState` resource wrapping `Game`, viewer player ID, and derived `GameSnapshot`
- [x] Implement `CardRegistryImpl` (maps card definition IDs to names for snapshot creation)
- [x] Add `AllowedActions` resource derived from `GetAllowedActions` query logic
- [x] Create startup system: create game, add players, assign bootstrap decks, start, advance to first main
- [x] Add 2D camera setup
- [x] Implement snapshot refresh system (runs when `GameState` is mutated)
- [x] Wire `main.rs` to use `EchomancyPlugin`
- [x] Verify: `cargo run` opens window, no panics, game state logged to console

#### Sub-phase 2: Card Rendering + Battlefield Layout (Medium) - DONE

**Objective**: Render cards as Bevy UI nodes and lay out both battlefields.

- [x] Create card UI component: `Node` with colored background (green=creature, brown=land), border
- [x] Add text children: card name, type line ("Creature" / "Land"), P/T for creatures
- [x] Implement tapped state: 90-degree rotation + alpha 0.85
- [x] Create `BattlefieldPlugin` with player battlefield row (bottom area)
- [x] Create opponent battlefield row (top area, visually distinct)
- [x] Spawn card entities from `GameSnapshot.private_player_state.battlefield` and `opponent_states[0].battlefield`
- [x] Implement despawn-and-rebuild when snapshot changes (simple approach for MVP)
- [x] Verify: cards appear on battlefield after playing a land via console/code

#### Sub-phase 3: Hand Display + Play Land Interaction (Medium)

**Objective**: Render the player's hand with overlap and enable land play via click.

- [ ] Create `HandPlugin` rendering hand cards in overlapping horizontal row
- [ ] Apply overlap offset (each card shifted left, similar to 60px in TS)
- [ ] Highlight playable lands with green border (using `AllowedActions` resource)
- [ ] Add click handler: `Interaction` component on hand cards
- [ ] On click of playable land: dispatch `Action::PlayLand`, apply to game, refresh snapshot
- [ ] Add `GameActionEvent` Bevy event for action dispatch
- [ ] Create system that processes `GameActionEvent`, applies to `GameState`, handles errors
- [ ] Verify: can click a land in hand, it moves to battlefield

#### Sub-phase 4: HUD -- Turn Info, Life, Counts, Priority, Buttons (Medium)

**Objective**: Render all informational displays and interactive buttons.

- [ ] Create `HudPlugin` with root layout node (fixed position overlay or flex column)
- [ ] Turn display: "Turn {N} - {phase} {step}" from `PublicGameState`
- [ ] Life totals: player life from `PrivatePlayerState`, opponent from `OpponentState`
- [ ] Opponent hand count from `OpponentState.hand_size`
- [ ] Graveyard counts: player `graveyard.len()`, opponent `graveyard.len()`
- [ ] Priority indicator: "Your Priority" (green) / "Opponent's Priority" (gray) from `priority_player_id`
- [ ] "Pass" button: sends `Action::AdvanceStep`, disabled when `!ui_hints.can_pass_priority`
- [ ] "End Turn" button: sends `Action::EndTurn`, disabled when `!ui_hints.can_pass_priority`
- [ ] Button visual states: enabled (clickable color) vs disabled (grayed out)
- [ ] Verify: all info displays correctly, buttons work, turn advances

#### Sub-phase 5: Error Display + Polish + QA (Small)

**Objective**: Error handling, visual polish, and full QA pass.

- [ ] Create `ErrorPlugin`: displays error messages in red text
- [ ] Error state: `Resource<Option<String>>` set by action handler on `GameError`
- [ ] Auto-dismiss after timeout OR click to dismiss
- [ ] Visual polish: consistent spacing, readable text sizes, clear layout hierarchy
- [ ] Ensure all 18 acceptance criteria pass
- [ ] Run `cargo test` (all tests pass)
- [ ] Run `cargo clippy` (no warnings)
- [ ] Code review with `mtg-code-reviewer`
- [ ] Final validation with `qa-validator`

**Blockers**: None
**Notes**:
- All domain logic exists in echomancy-core (566 tests, complete)
- GameSnapshot and AllowedActionsResult already provide all data the UI needs
- Using Bevy UI nodes (not sprites) for card rendering -- simplest approach, matches TS pattern
- Despawn-and-rebuild for UI updates is acceptable for MVP (no partial diffing needed)
- Player 1 is always the "viewer" (local player perspective)
- The bootstrap deck from `commands.rs` provides the starting cards (2 lands + 5 creatures)
