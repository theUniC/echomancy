# Turn Structure

The phases and steps of a Magic: The Gathering turn as implemented in Echomancy.

## Key Concepts

- **5 Phases, 12 Steps** - Beginning, First Main, Combat, Second Main, Ending
- **StepMachine** - Handles step progression and turn passing
- **Turn-Based Actions** - Some steps have automatic actions (untap, draw)
- **Priority** - Players get priority in most steps (not untap)
- **Timing Windows** - Specific steps restrict what can be played

## How It Works

### Turn Structure

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

### Step Constants

The Step object provides type-safe references: UNTAP, UPKEEP, DRAW, FIRST_MAIN, BEGINNING_OF_COMBAT, DECLARE_ATTACKERS, DECLARE_BLOCKERS, COMBAT_DAMAGE, END_OF_COMBAT, SECOND_MAIN, END_STEP, CLEANUP.

See `src/echomancy/domainmodel/game/Steps.ts`.

### Step Progression

StepMachine advances steps. Returns next step plus a flag indicating whether turn passes to next player (CLEANUP → UNTAP transition).

See `src/echomancy/domainmodel/game/StepMachine.ts`.

### Step Behaviors

**Beginning Phase:**
- **Untap** - Permanents untap (no priority)
- **Upkeep** - "At the beginning of your upkeep" triggers fire
- **Draw** - Active player draws one card

**Main Phases:**
- Play one land per turn (shared across both main phases)
- Cast sorcery-speed spells and creatures
- Full priority for instants and abilities

**Combat Phase:**
- **Beginning of Combat** - Last chance to tap creatures before attacks
- **Declare Attackers** - Active player chooses attackers (creatures tap unless vigilance)
- **Declare Blockers** - Defending player assigns blockers
- **Combat Damage** - Damage dealt
- **End of Combat** - Combat state resets, COMBAT_ENDED event emits

**Ending Phase:**
- **End Step** - "At the beginning of your end step" triggers fire
- **Cleanup** - Discard to hand size, "until end of turn" effects expire (normally no priority)

### Events Emitted During Turn

| Step Transition | Event |
|-----------------|-------|
| Any step start | STEP_STARTED |
| End of combat | COMBAT_ENDED |
| Creature attacks | CREATURE_DECLARED_ATTACKER |

See `docs/game-events.md` for event details.

### Extra Phases

Engine supports scheduling extra phases for effects like "take an extra combat phase after this one." Extra steps are inserted after current step sequence.

## Rules

- Lands can only be played during main phases (FIRST_MAIN or SECOND_MAIN)
- One land per turn across both main phases
- Sorceries and creatures cast during main phases only
- Instants cast whenever player has priority
- Untap step has no priority
- Turn passes to next player on CLEANUP → UNTAP transition
