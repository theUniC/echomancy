# CLAUDE.md

Read `AGENTS.md` first for commands, file locations, and critical rules.

This file defines the agent workflow for Claude Code.

## Decision Tree (MANDATORY)

Never skip this. For every request:

1. **Trivial?** (typo, format, single-line fix) → Implement directly, go to Phase 4
2. **Needs spec?** (new feature) → `mtg-spec-writer` → (if rules-heavy: `mtg-domain-expert`)
3. **Plan** → `tech-lead-strategist`
4. **Implement**:
   - Domain logic / game engine → `senior-backend-engineer`
   - Bevy UI with visual design needs → `tcg-ui-designer` → `ui-engineer`
   - Bevy UI without design needs → `ui-engineer`
5. **QA** → `cargo test` → `cargo clippy` → `mtg-code-reviewer` → `qa-validator`
6. **Finalize** → Update docs → Move spec to `done/` → `/git-workflow`

## Phase Details

### Phase 1: Specification
**When**: New feature needs requirements
**Agent**: `mtg-spec-writer`
**Output**: Spec in `docs/specs/backlog/`
**Validation**: Use `mtg-domain-expert` for rules-heavy features

### Phase 2: Planning
**When**: Spec ready in `active/`
**Agent**: `tech-lead-strategist`
**Output**: Implementation plan with tasks, agent assignments, and QA plan
**Required**: Add "Implementation Tracking" section to spec (see agent definition)

### Phase 3: Implementation
**When**: After planning
**Agents**: `senior-backend-engineer`, `ui-engineer`, `tcg-ui-designer`
**Rule**: ALL implementation through specialized agents. No exceptions.

### Phase 4: QA
**Required for all non-trivial work**:
1. `cargo test` - All tests pass
2. `cargo clippy` - No warnings
3. `mtg-code-reviewer` - Code quality + MTG rules
4. `qa-validator` - Verify ALL acceptance criteria, mark `[x]`

### Phase 5: Finalization
**Prerequisite**: `qa-validator` passed
1. Update `docs/` if architectural changes
2. Move spec `active/` → `done/`
3. Use `/git-workflow`

## Rust Best Practices (MANDATORY)

All agents MUST follow these when writing Rust code:

### Visibility
- **`pub(crate)`** by default for internal items
- **`pub`** only for the crate's external API
- Newtype inner fields are **private** — use `.as_str()` or accessor methods
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)

### Error Handling
- **`thiserror`** in `echomancy-core` (library) — typed, matchable errors
- **`anyhow`** in `echomancy-bevy` (binary) — ergonomic error propagation
- Never use `.unwrap()` outside tests — use `?` or explicit error handling

### Architecture
- **`echomancy-core`**: Pure Rust library, zero Bevy dependency. Domain model + application layer
- **`echomancy-bevy`**: Bevy binary. Wraps domain types as Resources/Components
- Domain services take `&Game` (read) and return results; caller applies mutations to `&mut Game`
- Use `[workspace.dependencies]` — all versions centralized in root `Cargo.toml`

### Idiomatic Rust
- Prefer `&str` in function parameters, `String` in return types and struct fields
- Use `impl Into<String>` for flexible constructors
- Derive `Debug, Clone, PartialEq, Eq, Hash` where appropriate
- Use exhaustive `match` instead of if/else chains
- Don't over-use `.clone()` — prefer references
- Don't create traits prematurely — start concrete, extract when needed
- No `Box<dyn Any>` — use enums for finite variants, trait objects for open extension

### Testing
- Unit tests: inline `#[cfg(test)] mod tests` in same file
- Integration tests: `crates/echomancy-core/tests/` directory
- Use `cargo test` to run all, `cargo test <name>` to filter
- Deterministic RNG with `rand::SeedableRng` for shuffling tests

### Bevy Patterns (for echomancy-bevy)
- Plugin architecture: one `EchomancyPlugin` composed of sub-plugins
- Game aggregate as Bevy `Resource`
- Player actions as Bevy `Event`s
- Use `States` enum for game phases, `StateScoped` for cleanup
- Systems constrained by State + SystemSet with `run_if` conditions

## Agent Reference

| Agent | Purpose |
|-------|---------|
| `mtg-spec-writer` | Write specifications |
| `mtg-domain-expert` | Validate MTG rules completeness |
| `tech-lead-strategist` | Plan implementation + QA |
| `tcg-ui-designer` | Visual design (layout, states, Bevy-native) |
| `ui-engineer` | Bevy UI implementation |
| `senior-backend-engineer` | Rust domain logic, DDD, game engine |
| `mtg-code-reviewer` | Code review (Rust quality + MTG rules) |
| `qa-validator` | Verify acceptance criteria (MANDATORY before done) |

**Skills**: `/subagent-driven-development`, `/brainstorming`, `/git-workflow`
