---
name: test-driven-development
description: Use when implementing any feature or bugfix, before writing implementation code (project)
---

# Test-Driven Development (TDD)

Write the test first. Watch it fail. Write minimal code to pass.

## Project Context

**Test helpers** (use these, don't reinvent):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    let (game, player1, player2) = create_started_game();
    let creature = create_test_creature(&player1.id);
    add_creature_to_battlefield(&mut game, &player1.id, creature);
}
```

**Always resolve stack before asserting:**
```rust
game.apply(Action::CastSpell { player_id, card_id, targets: vec![] })?;
resolve_stack(&mut game, &player2.id, &player1.id);
assert_eq!(game.get_stack().len(), 0);
```

**Run tests:**
```bash
cargo test -- <test_name>
```

## The Cycle: Red -> Green -> Refactor

### 1. RED: Write failing test

```rust
#[test]
fn creature_with_flying_can_only_be_blocked_by_creatures_with_flying_or_reach() {
    let (mut game, player1, player2) = create_started_game();
    let flyer = create_test_creature_with_keywords(&player1.id, vec![StaticAbility::Flying]);
    let ground_blocker = create_test_creature(&player2.id);

    add_creature_to_battlefield(&mut game, &player1.id, flyer.clone());
    add_creature_to_battlefield(&mut game, &player2.id, ground_blocker.clone());

    // Declare flyer as attacker
    game.apply(Action::DeclareAttackers {
        player_id: player1.id.clone(),
        attackers: vec![flyer.id.clone()],
    }).unwrap();

    // Try to block with ground creature - should fail
    let result = game.apply(Action::DeclareBlockers {
        player_id: player2.id.clone(),
        blockers: vec![(flyer.id.clone(), ground_blocker.id.clone())],
    });
    assert!(result.is_err());
}
```

**Run it. Confirm it fails for the right reason.**

### 2. GREEN: Minimal code to pass

Write the simplest code that makes the test pass. No extras.

### 3. REFACTOR: Clean up

Only after green. Keep tests passing.

### 4. Repeat

Next test for next behavior.

## Good Tests

- **One behavior per test** - If name has "and", split it
- **Clear name** - Describes what should happen
- **Real code** - Use actual `Game`, not mocks
- **Use helpers** - `create_started_game()`, `create_test_creature()`, etc.

## Red Flags

Stop and fix if:
- Test passes immediately (you're testing existing behavior)
- Test errors instead of fails (fix the error first)
- Writing code before test exists

## Checklist

Before done:
- [ ] Test existed and failed before implementation
- [ ] Test failed for the right reason
- [ ] Wrote minimal code to pass
- [ ] All tests pass: `cargo test`
