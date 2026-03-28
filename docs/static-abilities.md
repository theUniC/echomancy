# Static Abilities

Consultative keyword abilities that modify game rule checks and validations.

## Key Concepts

- **Consultative**: Checked at validation points, not applied as effects
- **Keywords**: Data on card definitions (Flying, Reach, Vigilance)
- **Local queries**: Each check queries the permanent directly, no global registry
- **No layers (MVP)**: Full 7-layer system deferred for simplicity
- **Type-safe**: Enum variants prevent typos, enable exhaustive matching

## How It Works

Static abilities are declared on card definitions as keyword constants. When the engine validates an action (e.g., declaring a blocker), it queries the relevant permanents for their keywords.

### MVP Keywords

| Keyword | Effect |
|---------|--------|
| Flying | Can only be blocked by creatures with Flying or Reach |
| Reach | Can block creatures with Flying |
| Vigilance | Doesn't tap when attacking |

**Why these three?** They modify exactly one rule each, don't interact with the stack, and require no replacement effects or dependencies.

### Evaluation Points

| Game Action | Keywords Consulted | Validation |
|-------------|-------------------|------------|
| Declare attacker | Vigilance | Skip tap if present |
| Declare blocker | Flying, Reach | If attacker has Flying, blocker needs Flying or Reach |

**Implementation**:
- Card definitions: `CardDefinition.static_abilities` in `crates/echomancy-core/src/domain/cards/`
- Validation: `game.declare_attacker()` and `game.declare_blocker()` in `crates/echomancy-core/src/domain/game/`
- Tests: Static ability tests in `crates/echomancy-core/src/domain/`

## Rules

### Design Principles

**Consultative vs Continuous**:
- Official rules: "This creature has +1/+1" creates effect object in layer 7c
- MVP: "Does this creature have Flying?" returns true/false at validation time

**Why no global registry?**
- Simpler: No separate data structure to maintain
- Correct: No risk of registry desync
- Testable: Each check isolated
- Fast: For MVP card count, direct queries beat registry overhead

**Why no layers?**
Layers (7 layers with sublayers, dependency resolution, timestamps) only needed when:
- Multiple effects modify same property
- Effects create circular dependencies
- Temporal ordering matters ("until end of turn")

MVP keywords don't have these requirements.

### Type Safety

Uses enum variants instead of strings:
```rust
// See StaticAbility enum in cards module
if has_static_ability(card, StaticAbility::Flying) { ... }
```

Benefits: exhaustive matching, compile-time checking, refactoring safety.
