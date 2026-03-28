# Testing Guide

Testing patterns and helpers for the Echomancy game engine.

## Key Concepts

- **Test Helpers** - Reusable functions in `#[cfg(test)]` modules for consistent setup
- **Arrange-Act-Assert** - Standard test structure (setup, action, verify)
- **Stack Resolution** - Must resolve the stack before asserting on spell effects
- **No Mocks** - Use actual Game struct, not mocks

## How It Works

**Running Tests**:
- `cargo test` - Run all tests
- `cargo test <pattern>` - Run tests matching pattern
- `cargo test -- --nocapture` - Show println output

**Before Committing**: Always run `cargo test && cargo clippy`

**Test Helpers** (see `#[cfg(test)]` modules for implementations):

**Game Setup**:
- `create_started_game()` - 2-player game in UNTAP step
- `create_game_in_main_phase()` - Game advanced to FIRST_MAIN
- `advance_to_step()` - Advance to any step

**Card Creation**:
- `create_test_creature()` - Basic creature
- `create_test_spell()` - Basic instant
- `create_creature_with_etb_trigger()` - Creature with ETB callback
- Themed helpers: `create_elvish_visionary()`, `create_llanowar_elves()`, etc.

**Zone Manipulation**:
- `add_creature_to_battlefield()` - Properly adds creature (fires ETB triggers)
- `add_spell_to_hand()` - Add spell to hand
- `add_creature_to_hand()` - Add creature to hand

**Combat**:
- `setup_creature_in_combat()` - Create creature and advance to DECLARE_ATTACKERS
- `setup_multiple_creatures()` - Add multiple creatures to battlefield

**Stack**:
- `resolve_stack()` - Both players pass, top item resolves
- `assert_spell_at()` - Type-safe stack inspection for spells
- `assert_ability_at()` - Type-safe stack inspection for abilities

**Extra Phases**:
- `schedule_extra_combat_phase()` - Schedule additional combat phase

## Rules

- Always use helpers instead of manual setup
- Never push directly to battlefield collections (use `add_creature_to_battlefield()`)
- Always resolve stack before asserting on spell effects
- Use `assert!(matches!(...))` or `assert_eq!` for assertions
- Follow Arrange-Act-Assert pattern
- One behavior per test

## Common Mistakes

- Constructing `Game` directly (use helpers)
- Pushing to battlefield without calling `enter_battlefield()`
- Asserting before resolving the stack
- Testing multiple unrelated behaviors in one test

**Test Organization**: Unit tests are inline `#[cfg(test)] mod tests` in the same file. Integration tests are in `crates/echomancy-core/tests/`.
