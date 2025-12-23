# Turn Structure

This document describes the phases and steps of a Magic: The Gathering turn as implemented in Echomancy.

## Turn Overview

A turn consists of 5 phases containing 12 steps total:

```
BEGINNING PHASE
├── Untap Step
├── Upkeep Step
└── Draw Step

FIRST MAIN PHASE
└── First Main Step

COMBAT PHASE
├── Beginning of Combat Step
├── Declare Attackers Step
├── Declare Blockers Step
├── Combat Damage Step
└── End of Combat Step

SECOND MAIN PHASE
└── Second Main Step

ENDING PHASE
├── End Step
└── Cleanup Step
```

## Step Constants

The Step constant object provides type-safe references to all steps: UNTAP, UPKEEP, DRAW, FIRST_MAIN, BEGINNING_OF_COMBAT, DECLARE_ATTACKERS, DECLARE_BLOCKERS, COMBAT_DAMAGE, END_OF_COMBAT, SECOND_MAIN, END_STEP, and CLEANUP.

## Step Progression

The StepMachine handles step advancement. It takes the current step and returns the next step, plus a flag indicating whether the turn should pass to the next player (which happens when transitioning from CLEANUP to UNTAP).

## Step Behaviors

### Beginning Phase

**Untap Step:** Active player's permanents untap. No player gets priority during this step - it's a turn-based action.

**Upkeep Step:** "At the beginning of your upkeep" triggers check here. Players get priority.

**Draw Step:** Active player draws a card. Players get priority after the draw.

### Main Phases

Both First Main and Second Main work the same way:
- Player can play one land per turn (across both main phases combined)
- Player can cast sorcery-speed spells and creatures
- Full priority available for instants and abilities

### Combat Phase

**Beginning of Combat:** "At the beginning of combat" triggers fire. Last chance to tap creatures before attacks are declared.

**Declare Attackers:** Active player chooses which creatures attack. Each attacking creature generates a CREATURE_DECLARED_ATTACKER event. Creatures tap when attacking (unless they have vigilance, which is not implemented in MVP).

**Declare Blockers:** Defending player assigns blockers. Not fully implemented in MVP.

**Combat Damage:** Damage is dealt. Damage system not implemented in MVP.

**End of Combat:** "At end of combat" triggers fire. Combat state resets. COMBAT_ENDED event emits.

### Ending Phase

**End Step:** "At the beginning of your end step" triggers fire. Last chance to act before cleanup.

**Cleanup Step:** Discard to hand size (not implemented in MVP). "Until end of turn" effects expire (not implemented). Normally no player gets priority.

## Land Playing Restrictions

Lands can only be played during main phases (FIRST_MAIN or SECOND_MAIN). Attempting to play a land at other times throws an error.

Players can only play one land per turn. The second land attempt in the same turn throws an error.

## Spell Timing

The MVP implements basic timing:
- Sorceries and creatures can be cast during main phases only
- Instants can be cast whenever the player has priority (but timing restrictions are simplified in MVP)

## Extra Phases

The engine supports scheduling extra phases. This is used for effects like "take an extra combat phase after this one." Extra steps are inserted into the turn after the current step sequence.

## Events Emitted During Turn

| Step Transition | Event |
|-----------------|-------|
| Any step start | STEP_STARTED |
| End of combat | COMBAT_ENDED |
| Creature attacks | CREATURE_DECLARED_ATTACKER |
