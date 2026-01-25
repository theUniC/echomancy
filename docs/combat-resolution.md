# Combat Resolution System

The combat system implements Magic's combat phases: declaring attackers and blockers, assigning damage, and destroying creatures with lethal damage.

## Key Concepts

- **Combat Phases** - DECLARE_ATTACKERS, DECLARE_BLOCKERS, COMBAT_DAMAGE, END_OF_COMBAT
- **Damage Assignment** - Simultaneous damage between creatures or to player
- **State-Based Actions** - Automatic creature destruction when damage >= toughness
- **Combat State** - Tracks attacking/blocking relationships, cleared at END_OF_COMBAT
- **Damage Tracking** - Accumulates per turn, cleared at CLEANUP step

## How It Works

See `src/echomancy/domainmodel/game/Game.ts` for implementation.

**Combat Flow**:
1. DECLARE_ATTACKERS: Active player declares attackers (must be untapped, become tapped)
2. DECLARE_BLOCKERS: Defending player assigns blockers (1-to-1 blocking only in MVP)
3. COMBAT_DAMAGE: Automatic damage assignment
   - Blocked attackers: attacker and blocker deal damage to each other (simultaneous)
   - Unblocked attackers: deal damage to defending player
4. State-Based Actions: Creatures with damage >= toughness are destroyed (moved to graveyard)
5. END_OF_COMBAT: Combat state cleared (isAttacking, blockingCreatureId, blockedBy)
6. CLEANUP: Damage cleared (damageMarkedThisTurn reset to 0)

**Damage Assignment**:
- Uses `getCurrentPower()` for damage amount
- Damage accumulates in `damageMarkedThisTurn` on each creature
- Compared to `getCurrentToughness()` for lethal damage checks

**Player Damage**:
- Starting life: 20
- Unblocked attackers reduce player life by their power
- No win/loss condition checked in MVP

## Rules

- Only untapped creatures can attack or block
- Attacking creatures become tapped when declared
- Creature can only attack once per turn
- Creature can only block one attacker per combat
- Each attacker can only be blocked by one creature (MVP: 1-to-1 only)
- Damage is simultaneous (both deal before either dies)
- Lethal damage = damage marked >= current toughness
- Combat state clears at END_OF_COMBAT
- Damage clears at CLEANUP (once per turn)
- Counters on creatures affect damage calculations

**Testing**: See `Game.combatResolution.test.ts` for full test suite.
