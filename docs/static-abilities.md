# Static Abilities System

This document describes the static ability system in Echomancy, focusing on the MVP implementation of consultative keywords.

## Overview

Static abilities are always-on permanent modifiers that affect game rule checks and validations. Unlike triggered or activated abilities, they do not use the stack and are not active effects—they are consulted when the engine evaluates specific rules.

## Design Philosophy

The static abilities system follows Echomancy's core principles:

1. **Consultative, not reactive**: Static abilities are checked at validation points, not applied as effects
2. **Local, not global**: Each check queries the relevant permanent directly
3. **Declarative**: Abilities are data on card definitions, not runtime behavior
4. **No layers (MVP)**: The full 7-layer system is intentionally deferred

This design keeps the engine simple and testable while supporting the most common keywords.

## MVP Scope

The MVP implements three consultative keywords:

- **Flying**: Can only be blocked by creatures with Flying or Reach
- **Reach**: Can block creatures with Flying
- **Vigilance**: Does not tap when declaring as an attacker

These keywords were chosen because they:
- Modify exactly one validation rule each
- Do not interact with the stack or priority
- Do not require replacement effects
- Do not create dependencies between permanents

## How It Works

### Card Definitions

Static abilities are declared on card definitions as an array of keyword constants. The type system ensures only valid keywords can be specified.

**Implementation reference:** See `CardDefinition.staticAbilities` in `src/echomancy/domainmodel/cards/CardDefinition.ts`

### Rule Checks

When the game engine needs to validate an action (e.g., declaring a blocker), it queries the relevant permanents for their static abilities. The check is performed inline at the validation point.

**Key principle:** The engine does not maintain a global registry of static abilities. Each check queries the permanent directly.

### Evaluation Points

Static abilities are consulted at these specific points:

| Game Action | Keywords Consulted | Validation Logic |
|-------------|-------------------|------------------|
| Declare attacker | Vigilance | If present, skip tap step |
| Declare blocker | Flying, Reach | If attacker has Flying, blocker must have Flying or Reach |

**Implementation reference:** See `Game.declareAttacker()` and `Game.declareBlocker()` in `src/echomancy/domainmodel/game/Game.ts`

## Architecture Decisions

### Why Consultative?

In the full Magic rules, static abilities create continuous effects that must be layered and resolved with dependency tracking. For the MVP, we use a simpler consultative model:

- **Continuous effects model** (official rules): "This creature has +1/+1" creates an effect object that applies in layer 7c
- **Consultative model** (MVP): "Does this creature have Flying?" returns true/false at validation time

The consultative model works for keywords that only affect binary yes/no decisions (can this block that? does this tap?).

### Why No Layers?

The full layer system (7 layers with sublayers, dependency resolution, timestamps) is complex and only necessary when:
- Multiple effects modify the same property
- Effects can create circular dependencies
- Temporal ordering matters ("until end of turn" vs permanent changes)

The MVP keywords do not have these requirements. When we add advanced keywords (buffs, ability gain/loss), we will implement layers.

### Why No Global Registry?

Each query checks the permanent directly instead of consulting a global registry because:

1. **Simplicity**: No need to maintain and update a separate data structure
2. **Correctness**: No risk of registry desync with card state
3. **Testability**: Each check is isolated and easy to test
4. **Performance**: For the MVP card count, direct queries are faster than registry overhead

When we add advanced features (lords affecting other creatures), we may introduce caching or indexing.

## Constants and Type Safety

The implementation uses typed constants instead of magic strings:

**Why constants?**
- IDE autocomplete support
- Compile-time type checking
- Refactoring safety (rename propagates)
- Runtime typo detection

**Implementation reference:** See `StaticAbilities` constant object in `src/echomancy/domainmodel/cards/CardDefinition.ts`

## Known Limitations (Intentional)

The MVP static abilities system explicitly does not support:

### 1. Ability Gain/Loss
Cards cannot dynamically gain or lose static abilities ("creature gains flying until end of turn"). This requires:
- Duration tracking system
- Layer application order
- Timestamp resolution

### 2. Continuous Effects
Static abilities cannot modify properties of other permanents ("other creatures you control get +1/+1"). This requires:
- Layer system (layers 1-7)
- Dependency resolution
- Affected object tracking

### 3. Advanced Keywords
Keywords not included in MVP:
- First strike / Double strike (require damage substeps)
- Trample (requires damage assignment ordering)
- Deathtouch (requires lethal damage redefinition)
- Menace (requires multiple blocker support)
- Lifelink (requires life gain events)
- Hexproof/Shroud/Protection (require targeting restrictions)
- Ward (requires cost payment on targeting)

Each of these has specific dependencies documented in ROADMAP.md.

### 4. Layer System
The official 7-layer system with dependency resolution is deferred. When implemented, it will:
- Determine which effects apply in which order
- Handle circular dependencies
- Resolve timestamp conflicts
- Support temporary vs permanent modifications

## Testing Strategy

Static abilities are tested through contract tests that verify rule checks in isolation:

**Test categories:**
1. Keyword presence tests (creature has ability → rule modified)
2. Keyword absence tests (creature lacks ability → normal rules)
3. Interaction tests (Flying + Reach, Vigilance + Flying)
4. Error case tests (invalid blocks throw correct errors)

**Test reference:** See `Game.staticAbilities.test.ts`

## Future Expansion

When expanding beyond the MVP:

### Phase 1: Additional Simple Keywords
Add keywords with similar characteristics (binary, local, no dependencies):
- Haste (can attack/activate same turn)
- Defender (cannot attack)

### Phase 2: Ability Gain/Loss
Implement duration tracking and temporary modifications:
- "Until end of turn" effects
- Aura/Equipment granting abilities
- Timestamp tracking

### Phase 3: Continuous Effects
Implement the layer system for:
- Lords and anthem effects
- P/T modifications
- Ability granting to other permanents
- Dependency resolution

### Phase 4: Complex Keywords
Add keywords requiring special systems:
- First strike (combat damage substeps)
- Trample (damage assignment order)
- Deathtouch (lethal damage rules)

## Integration with Other Systems

### Combat System
Flying and Reach integrate with blocker declaration validation. The combat damage resolution system is unaware of these keywords—it only cares whether a block was legally declared.

**Reference:** See `docs/combat-resolution.md`

### Creature State
Vigilance integrates with attacker declaration. The creature state system tracks tapped/attacking/attacked separately from static abilities.

**Reference:** See `docs/creature-stats.md`

### Type System
Static abilities use TypeScript union types and const assertions for maximum type safety without runtime overhead.

**Design principle:** Prefer compile-time guarantees over runtime checks.

## Common Pitfalls

### Don't: Check abilities globally
```typescript
// ❌ BAD: Global ability registry
const flyingCreatures = allCreatures.filter(hasFlying)
```

Query each permanent when needed:
```typescript
// ✅ GOOD: Query at validation point
if (hasStaticAbility(attacker, StaticAbilities.FLYING)) {
  // validate blocker has Flying or Reach
}
```

### Don't: Create effect objects
```typescript
// ❌ BAD: Treating keywords as effects
const flyingEffect = { type: "FLYING", applies: true }
```

Keywords are data on card definitions:
```typescript
// ✅ GOOD: Keywords as data
definition: { staticAbilities: [StaticAbilities.FLYING] }
```

### Don't: Use magic strings
```typescript
// ❌ BAD: String literals
if (hasStaticAbility(card, "FLYING")) { }
```

Use typed constants:
```typescript
// ✅ GOOD: Type-safe constants
if (hasStaticAbility(card, StaticAbilities.FLYING)) { }
```

## Summary

The static abilities MVP provides a minimal, correct implementation of consultative keywords. It prioritizes:

- **Correctness**: Implements actual Magic rules faithfully
- **Simplicity**: No premature abstraction or over-engineering
- **Testability**: Each keyword is independently verifiable
- **Extensibility**: Clear path to full layer system when needed

The intentional limitations are documented and have clear expansion paths defined in ROADMAP.md.
