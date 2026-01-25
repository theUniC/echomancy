# Stack and Priority

The stack resolves spells and abilities in Last In, First Out (LIFO) order. Priority determines who can act and when.

## Key Concepts

- **The Stack** - LIFO queue for spells and abilities awaiting resolution
- **Priority** - Permission to take actions (cast, activate, or pass)
- **Priority Flow** - Active player gets priority first, then passes around table
- **Resolution** - All players passing in succession causes top item to resolve
- **Last Known Information** - Abilities resolve even if source is destroyed

## How It Works

See `src/echomancy/domainmodel/game/Game.ts` for implementation.

**Stack Items**:
- **Spells**: Card being cast with controller and targets. Resolves by executing effect and moving card to graveyard or battlefield.
- **Activated Abilities**: Source permanent ID, effect, controller, targets. Uses Last Known Information (resolves even if source leaves battlefield).
- **Triggered Abilities**: Defined but not yet used in MVP. Triggers currently execute immediately.

**Priority Flow**:
1. Player casts spell or activates ability (item goes on stack)
2. Active player receives priority
3. Players can respond or pass
4. All players pass in succession → top item resolves
5. Active player receives priority again
6. Repeat until stack empty and all players pass

**Resolution Order**: Items resolve in reverse order of addition (last added resolves first).

**END_TURN Shortcut**: Records auto-pass intent. Player automatically passes priority until turn ends. Opponent can still respond normally. Auto-pass clears at start of next turn (UNTAP step).

## Rules

- Stack uses LIFO ordering
- Both players passing in succession resolves top item
- Active player always receives priority first after resolution
- Actions that use stack: CAST_SPELL, ACTIVATE_ABILITY
- Actions that don't use stack: PLAY_LAND, combat declarations, step advancement, passing priority
- Last Known Information applies to activated abilities
- Auto-pass intent is per-player and clears each turn

**Testing**: See `Game.priorityAndStackResolution.test.ts` for comprehensive coverage.
