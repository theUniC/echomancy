# UI MVP Specification

This document specifies the minimal playable UI for Echomancy. It covers component architecture, state management, game loop integration, and detailed UI areas with wireframe descriptions.

---

## 1. Overview

### 1.1 Purpose

The UI MVP enables two players to play a complete game of Echomancy using the core engine. It provides:

- Visual representation of game state
- Input mechanisms for all player actions
- Clear feedback on game progression and legal moves

### 1.2 Design Goals

1. **Engine-driven**: UI reflects GameSnapshot only; never infers rules
2. **Clarity**: Game state is always unambiguous to players
3. **Responsiveness**: Actions provide immediate visual feedback
4. **Simplicity**: Minimal visual complexity for MVP

### 1.3 Scope

**In Scope:**
- Zone UI: Hand, Battlefield, Graveyard
- Stack UI: Visible stack with order, source, controller
- Priority UI: Active player indicator, pass priority
- Turn and Phase UI: Turn owner, phase/step display
- Combat UI: Attacker selection, blocker selection (1-to-1)
- Target Selection UI: Valid target highlighting, confirmation/cancellation
- Action UI: Play land, Cast spell, Activate ability, Declare attacker/blocker, End turn

**Out of Scope:**
- Deck builder
- Matchmaking
- Replays
- Animations
- Sound
- Mobile-specific layouts

### 1.4 Tech Stack

- Next.js 16
- React 19
- TypeScript (strict mode)
- Tailwind CSS v4

---

## 2. Architecture

### 2.1 Core Principle: Engine as Authority

The UI follows a strict unidirectional data flow:

```
User Input
    |
    v
Action Dispatch --> Game.apply(action)
    |
    v
Game Engine (validates, executes)
    |
    v
game.exportState()
    |
    v
createGameSnapshot(export, viewerId, registry)
    |
    v
GameSnapshot (player-relative, filtered)
    |
    v
React Components (render)
```

**Critical Rules:**
- UI NEVER mutates game state directly
- UI NEVER infers rule legality
- UI ALWAYS asks engine for allowed actions
- UI ONLY renders from GameSnapshot

### 2.2 Component Architecture

```
src/
  app/
    game/
      [gameId]/
        page.tsx           # Game page (Server Component)

  components/
    game/
      GameBoard.tsx        # Main game container (Client Component)

      zones/
        Hand.tsx           # Player's hand zone
        Battlefield.tsx    # Battlefield zone
        Graveyard.tsx      # Graveyard zone (expandable)
        Stack.tsx          # Stack display

      cards/
        Card.tsx           # Base card component
        CardInHand.tsx     # Card in hand (playable)
        CardOnBattlefield.tsx  # Card on battlefield
        CardInStack.tsx    # Card/ability on stack
        CardInGraveyard.tsx    # Card in graveyard

      controls/
        TurnDisplay.tsx    # Turn/phase/step indicator
        PriorityIndicator.tsx  # Who has priority
        ActionBar.tsx      # Primary action buttons
        ManaPoolDisplay.tsx    # Mana pool visualization
        LifeDisplay.tsx    # Life total display

      combat/
        CombatOverlay.tsx  # Combat phase overlay
        AttackerSelector.tsx   # Attacker selection UI
        BlockerSelector.tsx    # Blocker assignment UI

      targeting/
        TargetingOverlay.tsx   # Target selection overlay
        TargetHighlight.tsx    # Highlight valid targets

      modals/
        ConfirmationModal.tsx  # Generic confirmation
        GraveyardViewer.tsx    # Full graveyard view

  hooks/
    useGameState.ts        # Game state subscription
    useGameActions.ts      # Action dispatch helpers
    useTargeting.ts        # Target selection state
    useCombat.ts           # Combat selection state
    useAllowedActions.ts   # Query allowed actions

  lib/
    game/
      GameClient.ts        # Client-side game wrapper
      CardRegistry.ts      # Card name resolution
      ActionDispatcher.ts  # Action validation + dispatch
```

### 2.3 Component Hierarchy

```
GameBoard (Client)
  |
  +-- OpponentArea
  |     +-- LifeDisplay
  |     +-- ManaPoolDisplay (opponent's visible pool)
  |     +-- HandIndicator (card count only)
  |     +-- Battlefield (opponent's)
  |     +-- Graveyard (opponent's, collapsed)
  |
  +-- GameInfo
  |     +-- TurnDisplay
  |     +-- PriorityIndicator
  |     +-- Stack
  |
  +-- PlayerArea
  |     +-- Battlefield (player's)
  |     +-- Hand
  |     +-- Graveyard (player's, collapsed)
  |     +-- ManaPoolDisplay
  |     +-- LifeDisplay
  |
  +-- ActionBar
  |     +-- PassPriorityButton
  |     +-- EndTurnButton
  |     +-- ConfirmBlockersButton (combat only)
  |
  +-- Overlays (conditional)
        +-- TargetingOverlay
        +-- CombatOverlay
        +-- ConfirmationModal
```

---

## 3. State Management

### 3.1 State Categories

**Server State (Engine-Owned):**
- Complete game state
- Rule validation
- Action execution
- All accessed via GameSnapshot

**Client State (UI-Owned):**
- UI mode (normal, targeting, combat)
- Selected targets (before confirmation)
- Expanded/collapsed zones
- Hover states
- Pending action context

### 3.2 State Flow

```typescript
// Primary game state hook
type GameStateContext = {
  snapshot: GameSnapshot | null
  isLoading: boolean
  error: Error | null
  refresh: () => Promise<void>
}

// UI mode state
type UIMode =
  | { type: 'NORMAL' }
  | { type: 'TARGETING', pendingAction: PendingTargetedAction }
  | { type: 'DECLARING_ATTACKERS' }
  | { type: 'DECLARING_BLOCKERS' }
  | { type: 'CONFIRMING', action: Actions }

// Pending action requiring targets
type PendingTargetedAction = {
  actionType: 'CAST_SPELL' | 'ACTIVATE_ABILITY'
  sourceCardId: string
  requiredTargetCount: number
  validTargetIds: string[]
  selectedTargetIds: string[]
}
```

### 3.3 State Management Approach

Use React 19 features with minimal external state:

1. **Server Components** for initial page load
2. **Client Components** for interactive game board
3. **React Context** for game state distribution
4. **Local state** for UI-only concerns (hover, expand/collapse)
5. **No external state library** - React Context + hooks sufficient for MVP

```typescript
// GameContext.tsx
type GameContextValue = {
  snapshot: GameSnapshot
  allowedActions: AllowedAction[]
  dispatch: (action: Actions) => Promise<void>
}

// UIStateContext.tsx
type UIStateContextValue = {
  mode: UIMode
  setMode: (mode: UIMode) => void
  selectedTargets: string[]
  toggleTarget: (targetId: string) => void
  clearTargets: () => void
}
```

---

## 4. Game Loop Integration

### 4.1 Action Flow

```typescript
// ActionDispatcher.ts
class ActionDispatcher {
  constructor(
    private game: Game,
    private viewerId: string,
    private cardRegistry: CardRegistry,
    private onStateChange: (snapshot: GameSnapshot) => void
  ) {}

  async dispatch(action: Actions): Promise<void> {
    // 1. Validate action is allowed
    const allowed = this.game.getAllowedActionsFor(action.playerId)
    if (!allowed.includes(action.type)) {
      throw new ActionNotAllowedError(action.type)
    }

    // 2. Apply action to engine
    this.game.apply(action)

    // 3. Export new state
    const exported = this.game.exportState()

    // 4. Create snapshot for viewer
    const snapshot = createGameSnapshot(
      exported,
      this.viewerId,
      this.cardRegistry
    )

    // 5. Notify UI
    this.onStateChange(snapshot)
  }
}
```

### 4.2 Allowed Actions Query

```typescript
// useAllowedActions.ts
function useAllowedActions(
  game: Game,
  playerId: string
): AllowedAction[] {
  // Query engine for legal actions
  return game.getAllowedActionsFor(playerId)
}
```

### 4.3 Polling vs Push

**MVP Approach: Polling with optimistic updates**

For local two-player gameplay on same device:
- Game state updates immediately on action
- No network latency concerns
- Simple synchronous flow

Future consideration for networked play:
- WebSocket for push updates
- Optimistic UI with rollback

---

## 5. UI Areas

### 5.1 Hand Zone

**Purpose:** Display player's hand cards with play/cast actions.

**Wireframe:**
```
+------------------------------------------------------------------+
|  [Card 1]  [Card 2]  [Card 3]  [Card 4]  [Card 5]               |
|   Forest    Llanowar   Giant    Shock     Mountain              |
|             Elves     Growth                                     |
+------------------------------------------------------------------+
```

**Behavior:**
- Cards displayed horizontally, left-aligned
- Scrollable if more than fit
- Hover: slight lift, show full card details
- Click on land (during main phase): PLAY_LAND action
- Click on spell (during main phase, with priority): initiate CAST_SPELL
- Disabled appearance when action not allowed
- Card count shown for opponent's hand (not cards)

**Visual States:**
- Playable: normal appearance, cursor pointer
- Not playable: dimmed, cursor not-allowed
- Selected: highlighted border

**Data Source:**
```typescript
snapshot.privatePlayerState.hand // CardSnapshot[]
snapshot.opponentStates[0].handSize // number (not cards)
```

### 5.2 Battlefield Zone

**Purpose:** Display permanents in play for both players.

**Wireframe:**
```
+------------------------------------------------------------------+
| OPPONENT'S BATTLEFIELD                                            |
|  [Grizzly Bears]  [Forest]  [Mountain]                           |
|     2/2 (T)          (T)                                         |
+------------------------------------------------------------------+
| YOUR BATTLEFIELD                                                  |
|  [Llanowar Elves]  [Giant Spider]  [Forest]  [Forest]           |
|     1/1               2/4            (T)                         |
+------------------------------------------------------------------+
```

**Layout:**
- Split horizontally: opponent top, player bottom
- Creatures on left, non-creature permanents on right
- Tapped cards rotated 90 degrees clockwise
- Counters displayed as badges

**Card Display:**
- Name
- Power/Toughness (creatures only)
- Tapped indicator
- Counter badges (+1/+1 counter count)
- Attacking indicator (red border during combat)
- Blocking indicator (blue border during combat)

**Behavior:**
- Click creature during DECLARE_ATTACKERS: toggle attack declaration
- Click creature during DECLARE_BLOCKERS: initiate block assignment
- Click permanent with activated ability: initiate ACTIVATE_ABILITY
- Hover: show full card details

**Data Source:**
```typescript
snapshot.privatePlayerState.battlefield // CardSnapshot[]
snapshot.opponentStates[0].battlefield // CardSnapshot[]
```

### 5.3 Graveyard Zone

**Purpose:** Display cards in graveyard (public information).

**Wireframe (collapsed):**
```
+------------------+
| Graveyard (3)    |
|  [top card img]  |
+------------------+
```

**Wireframe (expanded):**
```
+------------------------------------------------------------------+
| Graveyard (3)                                           [Close]  |
|  [Lightning Bolt]  [Shock]  [Mountain]                          |
+------------------------------------------------------------------+
```

**Behavior:**
- Collapsed by default, shows count + top card
- Click to expand modal with full graveyard view
- Cards ordered newest first (top of graveyard)
- No interactive actions in MVP

**Data Source:**
```typescript
snapshot.privatePlayerState.graveyard // CardSnapshot[]
snapshot.opponentStates[0].graveyard // CardSnapshot[]
```

### 5.4 Stack Zone

**Purpose:** Display spells and abilities waiting to resolve.

**Wireframe:**
```
+----------------------------------+
| STACK                            |
| +------------------------------+ |
| | 1. Lightning Bolt            | |
| |    -> Grizzly Bears          | |
| |    Controller: You           | |
| +------------------------------+ |
| | 2. Giant Growth              | |
| |    -> Llanowar Elves         | |
| |    Controller: Opponent      | |
| +------------------------------+ |
+----------------------------------+
```

**Layout:**
- Vertical list, top of stack at top
- Each item shows:
  - Source card/ability name
  - Targets (if any)
  - Controller
- Index number (1 = top, resolves first)

**Behavior:**
- No direct interaction in MVP
- Visual indicator when stack is non-empty
- Animated appearance on stack push (future)

**Data Source:**
```typescript
snapshot.visibleStack.items // StackItemSnapshot[]
// items[0] is TOP of stack
```

### 5.5 Turn and Phase Display

**Purpose:** Show current turn, phase, and step.

**Wireframe:**
```
+------------------------------------------+
| Turn 3 - YOUR TURN                       |
| Phase: Combat | Step: Declare Attackers  |
+------------------------------------------+
```

**Display Elements:**
- Turn number
- Active player indicator ("YOUR TURN" / "OPPONENT'S TURN")
- Current phase name
- Current step name

**Phase/Step Mapping:**
| Phase | Steps |
|-------|-------|
| Beginning | Untap, Upkeep, Draw |
| Precombat Main | First Main |
| Combat | Beginning of Combat, Declare Attackers, Declare Blockers, Combat Damage, End of Combat |
| Postcombat Main | Second Main |
| Ending | End Step, Cleanup |

**Data Source:**
```typescript
snapshot.publicGameState.turnNumber
snapshot.publicGameState.currentPlayerId
snapshot.publicGameState.currentPhase
snapshot.publicGameState.currentStep
```

### 5.6 Priority Indicator

**Purpose:** Show which player has priority and can act.

**Wireframe:**
```
+---------------------------+
| PRIORITY: You             |
| [Pass Priority] [Actions] |
+---------------------------+
```

**States:**
- "You have priority" - green indicator
- "Opponent has priority" - red indicator, waiting state
- "No priority" - gray, during non-priority steps (Untap, Cleanup)

**Behavior:**
- Pulse/glow when player has priority
- Show available actions when player has priority
- Disable action buttons when opponent has priority

**Data Source:**
```typescript
snapshot.publicGameState.priorityPlayerId
snapshot.uiHints.canPassPriority
```

### 5.7 Combat UI

**Purpose:** Manage attacker and blocker declarations.

#### 5.7.1 Attacker Selection

**Active during:** DECLARE_ATTACKERS step, active player only

**Wireframe:**
```
+------------------------------------------------------------------+
| DECLARE ATTACKERS                                    [Confirm]    |
+------------------------------------------------------------------+
| YOUR BATTLEFIELD                                                  |
|  [*Llanowar Elves*]  [Giant Spider]  [Forest]  [Forest]          |
|     1/1 ATTACKING       2/4            (T)                        |
+------------------------------------------------------------------+
```

**Behavior:**
- Click untapped creature: toggle attacking state
- Attacking creatures get red highlight/border
- Creatures tap when declared as attackers (unless Vigilance)
- "Confirm" button commits attackers (advances step)
- Can declare zero attackers (skip combat)

**Validation (from engine):**
- Only untapped creatures can attack
- Creature cannot have attacked this turn
- Creatures with summoning sickness cannot attack (future)

#### 5.7.2 Blocker Selection

**Active during:** DECLARE_BLOCKERS step, defending player only

**Wireframe:**
```
+------------------------------------------------------------------+
| DECLARE BLOCKERS                                     [Confirm]    |
+------------------------------------------------------------------+
| ATTACKING CREATURES                                               |
|  [Grizzly Bears]  [Elvish Mystic]                                |
|     2/2 ATTACKING    1/1 ATTACKING                               |
+------------------------------------------------------------------+
| YOUR CREATURES (click to assign as blocker)                       |
|  [Giant Spider]  [Llanowar Elves]                                |
|     2/4              1/1 (T)                                      |
+------------------------------------------------------------------+
```

**Behavior:**
1. Click your untapped creature: select as potential blocker
2. Click attacking creature: assign blocker to that attacker
3. Blocking relationship displayed with connecting line/highlight
4. "Confirm" commits blockers (advances step)
5. Can declare zero blockers

**Validation (from engine):**
- Only untapped creatures can block
- Creature can only block one attacker (MVP)
- Each attacker can only be blocked by one creature (MVP)
- Flying creatures can only be blocked by Flying/Reach

**Data Source:**
```typescript
// Attacking creatures
snapshot.uiHints.highlightedAttackers // string[] of instanceIds

// Combat state per card
card.combatState.isAttacking
card.combatState.isBlocking
card.combatState.blockedBy // string[]
card.combatState.blocking // string[]
```

### 5.8 Target Selection UI

**Purpose:** Select valid targets for spells and abilities.

**Wireframe:**
```
+------------------------------------------------------------------+
| SELECT TARGET FOR: Lightning Bolt              [Cancel] [Confirm] |
| Choose 1 target creature or player                                |
+------------------------------------------------------------------+
| VALID TARGETS (highlighted)                                       |
|  [*Grizzly Bears*]  [Elvish Mystic]  [OPPONENT]                  |
|     SELECTED                                                      |
+------------------------------------------------------------------+
```

**Behavior:**
1. Initiate: User clicks spell/ability that requires targets
2. Highlight: Valid targets get visual highlight
3. Select: Click valid target to select/deselect
4. Confirm: Click confirm when required targets selected
5. Cancel: Abort the action, return to normal state

**UI Mode Flow:**
```typescript
// User clicks Lightning Bolt in hand
setMode({
  type: 'TARGETING',
  pendingAction: {
    actionType: 'CAST_SPELL',
    sourceCardId: 'bolt-instance-1',
    requiredTargetCount: 1,
    validTargetIds: ['creature-1', 'creature-2', 'opponent-id'],
    selectedTargetIds: []
  }
})

// User clicks valid target
toggleTarget('creature-1')

// User clicks Confirm
dispatch({
  type: 'CAST_SPELL',
  playerId: viewerId,
  cardId: 'bolt-instance-1',
  targets: [{ kind: 'CARD', cardId: 'creature-1' }]
})
setMode({ type: 'NORMAL' })
```

**MVP Limitation:**
The engine's targeting system is simplified. Valid targets must be queried from the engine or card definition. For MVP, the UI may hardcode common targeting patterns.

### 5.9 Action Bar

**Purpose:** Primary action buttons for common game actions.

**Wireframe:**
```
+------------------------------------------------------------------+
| [Pass Priority]  [End Turn]                                       |
+------------------------------------------------------------------+
```

**Buttons:**

| Button | Visible When | Action |
|--------|--------------|--------|
| Pass Priority | Player has priority AND stack is non-empty | PASS_PRIORITY |
| End Turn | Player is active player | END_TURN |
| Confirm Attackers | DECLARE_ATTACKERS step | Implicit ADVANCE_STEP |
| Confirm Blockers | DECLARE_BLOCKERS step | Implicit ADVANCE_STEP |

**Data Source:**
```typescript
snapshot.uiHints.canPassPriority
snapshot.publicGameState.currentPlayerId === viewerId
snapshot.publicGameState.currentStep
```

### 5.10 Mana Pool Display

**Purpose:** Show available mana for the player.

**Wireframe:**
```
+----------------------------------+
| Mana: W:2 U:0 B:0 R:1 G:3 C:0    |
+----------------------------------+
```

**Display:**
- Six mana symbols with counts
- Only show non-zero counts (optional)
- Color-coded symbols (W=white, U=blue, B=black, R=red, G=green, C=gray)

**Behavior:**
- Read-only display
- Updates immediately after mana-producing abilities

**Data Source:**
```typescript
snapshot.privatePlayerState.manaPool // ManaPoolExport
```

### 5.11 Life Display

**Purpose:** Show life totals for both players.

**Wireframe:**
```
+------------------+
| Opponent: 17     |
| You: 20          |
+------------------+
```

**Data Source:**
```typescript
snapshot.privatePlayerState.lifeTotal
snapshot.opponentStates[0].lifeTotal
```

---

## 6. Interaction Patterns

### 6.1 Card Interactions

| Location | Click Action | Condition |
|----------|--------------|-----------|
| Hand (Land) | PLAY_LAND | Main phase, no land played |
| Hand (Spell) | Initiate CAST_SPELL | Main phase, has priority |
| Battlefield (Creature) | DECLARE_ATTACKER | Declare Attackers step, untapped |
| Battlefield (Creature) | Initiate DECLARE_BLOCKER | Declare Blockers step, untapped |
| Battlefield (Permanent) | ACTIVATE_ABILITY | Has ability, can pay cost |

### 6.2 Targeting Flow

```
1. User initiates action (click card in hand)
   |
   v
2. UI enters TARGETING mode
   - Query valid targets from engine/card definition
   - Highlight valid targets
   - Dim invalid targets
   |
   v
3. User selects targets (click highlighted items)
   - Track selected targets locally
   - Update visual selection state
   |
   v
4. User confirms or cancels
   - Confirm: dispatch action with targets
   - Cancel: exit TARGETING mode, restore normal state
```

### 6.3 Combat Flow

```
BEGINNING_OF_COMBAT
  |
  v
DECLARE_ATTACKERS (active player)
  - UI shows attacker selection overlay
  - Player clicks creatures to attack
  - Player clicks "Confirm" to proceed
  |
  v
DECLARE_BLOCKERS (defending player)
  - UI shows blocker selection overlay
  - Player clicks creatures to block
  - Player clicks attackers to assign blockers
  - Player clicks "Confirm" to proceed
  |
  v
COMBAT_DAMAGE (automatic)
  - Engine resolves damage
  - UI updates life totals
  - UI shows destroyed creatures moving to graveyard
```

---

## 7. Visual Design Guidelines

### 7.1 Layout

- Full viewport height game board
- Horizontal split: opponent area (top), game info (middle), player area (bottom)
- Responsive but desktop-first for MVP

### 7.2 Colors

| Element | Color |
|---------|-------|
| Player's side | Subtle green tint |
| Opponent's side | Subtle red tint |
| Priority indicator (yours) | Bright green |
| Priority indicator (opponent) | Gray/dim |
| Stack items | Yellow/gold highlight |
| Valid targets | Green glow |
| Selected targets | Blue border |
| Attacking creatures | Red glow |
| Blocking creatures | Blue glow |
| Tapped permanents | Rotated + dimmed |

### 7.3 Card Appearance

Minimal card representation for MVP:
- Name (prominently displayed)
- Type icons (creature, land, etc.)
- Power/Toughness (bottom right for creatures)
- Mana cost (top right)
- Tap indicator (rotated 90 degrees)
- Counter badges (overlaid numbers)

### 7.4 Accessibility Considerations

- Minimum touch target size: 44x44 pixels
- Color not sole indicator (use icons/text)
- Keyboard navigation for actions (future)
- Screen reader labels for interactive elements (future)

---

## 8. Error Handling

### 8.1 Action Errors

When an action fails (engine throws error):

1. Display error message to user
2. Do not change game state
3. Allow user to retry or cancel

```typescript
try {
  await dispatch(action)
} catch (error) {
  if (error instanceof GameError) {
    showErrorToast(error.message)
  }
}
```

### 8.2 Common Error States

| Error | User Message |
|-------|--------------|
| InvalidPlayLandStepError | "Can only play lands during main phase" |
| LandLimitExceededError | "Already played a land this turn" |
| InvalidPlayerActionError | "Not your turn / You don't have priority" |
| TappedCreatureCannotAttackError | "Tapped creatures cannot attack" |
| CannotBlockFlyingCreatureError | "Only creatures with Flying or Reach can block this creature" |

---

## 9. Implementation Phases

### Phase 1: Foundation (Week 1)

**Goal:** Render static game state, no interactions

**Tasks:**
1. Set up project structure
2. Implement CardRegistry
3. Create GameContext provider
4. Build Card component (base)
5. Build Hand zone (read-only)
6. Build Battlefield zone (read-only)
7. Build Graveyard zone (collapsed only)
8. Build TurnDisplay component
9. Build LifeDisplay component
10. Build ManaPoolDisplay component

**Deliverable:** Game state visible but not interactive

### Phase 2: Basic Actions (Week 2)

**Goal:** Play lands, cast spells, pass priority

**Tasks:**
1. Implement ActionDispatcher
2. Create useGameActions hook
3. Build ActionBar (Pass Priority, End Turn)
4. Make Hand cards clickable (PLAY_LAND)
5. Implement basic CAST_SPELL (no targets)
6. Build Stack display
7. Build PriorityIndicator
8. Handle action errors with toasts

**Deliverable:** Can play lands, cast non-targeted spells, pass turns

### Phase 3: Targeting System (Week 3)

**Goal:** Target selection for spells and abilities

**Tasks:**
1. Implement UIStateContext for mode management
2. Build TargetingOverlay component
3. Implement target highlighting
4. Create useTargeting hook
5. Wire CAST_SPELL with targets
6. Implement ACTIVATE_ABILITY with targets
7. Build Cancel/Confirm flow

**Deliverable:** Can cast targeted spells, activate abilities

### Phase 4: Combat System (Week 4)

**Goal:** Full combat flow

**Tasks:**
1. Build CombatOverlay component
2. Implement AttackerSelector
3. Implement BlockerSelector
4. Create useCombat hook
5. Wire DECLARE_ATTACKER action
6. Wire DECLARE_BLOCKER action
7. Display combat damage results
8. Handle creature destruction visually

**Deliverable:** Full combat with attackers and blockers

### Phase 5: Polish and Edge Cases (Week 5)

**Goal:** Complete MVP, handle edge cases

**Tasks:**
1. Expand graveyard modal
2. Add visual feedback for all actions
3. Handle all error states
4. Test two-player flow end-to-end
5. Responsive layout adjustments
6. Performance optimization
7. Code cleanup and documentation

**Deliverable:** Complete UI MVP

---

## 10. Testing Strategy

### 10.1 Unit Tests

- CardRegistry: name resolution
- GameSnapshot creation: verify visibility filtering
- Action dispatch: verify actions reach engine

### 10.2 Component Tests

- Each zone renders correctly from snapshot data
- Interactive elements respond to clicks
- Mode transitions work correctly

### 10.3 Integration Tests

- Full game flow: start to finish
- Combat flow: attackers to damage
- Target selection: spell to resolution

### 10.4 Manual Test Scenarios

1. Play land during main phase
2. Cast creature, resolve, appears on battlefield
3. Activate mana ability, mana pool updates
4. Cast targeted spell, select target, resolve
5. Declare attackers, confirm
6. Declare blockers, confirm
7. Combat damage resolves, creatures die
8. End turn, opponent's turn begins
9. Pass priority back and forth
10. Stack resolves in correct order

---

## 11. Open Questions

### 11.1 Resolved

- **State management:** React Context + hooks (no external library)
- **Targeting source:** Query engine for allowed actions, derive valid targets

### 11.2 Unresolved (Deferred)

- **Multiplayer networking:** Out of scope for MVP
- **Hot seat vs split screen:** MVP assumes single screen, players take turns
- **Timer/clock:** Not in MVP
- **Undo support:** Not in MVP

---

## 12. Dependencies

### 12.1 Engine Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| GameSnapshot | UI data contract | Implemented |
| Game.getAllowedActionsFor() | Query legal actions | Implemented |
| Game.apply() | Execute actions | Implemented |
| Game.exportState() | Get raw state | Implemented |
| createGameSnapshot() | Create player view | Implemented |

### 12.2 External Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| next | 16.x | Framework |
| react | 19.x | UI library |
| typescript | 5.x | Type safety |
| tailwindcss | 4.x | Styling |

---

## 13. Glossary

| Term | Definition |
|------|------------|
| GameSnapshot | Player-relative, visibility-filtered game state for rendering |
| Priority | Permission to take actions; alternates between players |
| Stack | LIFO queue of spells and abilities waiting to resolve |
| Active Player | Player whose turn it is |
| Defending Player | Opponent of the active player during combat |
| MVP | Minimum Viable Product |

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-29 | Initial specification |
