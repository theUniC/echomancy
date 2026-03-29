# Agent Instructions for Echomancy

TCG engine built with Rust and Bevy 0.18 game engine. Distributed as a native compiled binary.

## Architecture

```
Cargo.toml                    (workspace root)
crates/
  echomancy-core/             (lib — pure Rust, no Bevy dependency)
    src/domain/               (game engine: rules, entities, services)
    src/prelude.rs            (common re-exports)
  echomancy-bevy/             (bin — Bevy 0.18, UI, rendering)
    src/main.rs               (app entry point)
```

## First Action

Read `docs/README.md` before doing anything else. The documentation explains architectural decisions and constraints you must follow.

## Critical Rules

1. **English only** - All code, docs, commits, tests, error messages. No exceptions.
2. **Read docs before coding** - Read relevant `docs/*.md` files before any implementation.
3. **Update docs after coding** - Keep documentation synchronized with the engine.
4. **Ask before commit/push** - Always get explicit user confirmation first.
5. **Use workspace dependencies** - All versions in root `Cargo.toml` `[workspace.dependencies]`.
6. **Private by default** - Use `pub(crate)` for internal items, `pub` only for external API.

## Backlog Workflow

Specs location: `docs/specs/` with three folders: `backlog/` → `active/` → `done/`

**Find next task:**
1. Open `docs/specs/BACKLOG.md`
2. Find first item with status `TODO`
3. Spec file is in `docs/specs/backlog/`

**Start work:**
1. Update `BACKLOG.md`: change status to `IN PROGRESS`
2. Move spec from `backlog/` to `active/`

**Complete work:**
1. Update `BACKLOG.md`: change status to `DONE`, unblock dependent items
2. Move spec from `active/` to `done/`

## Task Workflow

1. **Read docs** - Read `docs/README.md` and relevant files
2. **Understand** - Read relevant source files before writing code
3. **Check patterns** - Look at existing similar code/tests
4. **Implement** - Write the code
5. **Test** - Run `cargo test`
6. **Lint** - Run `cargo clippy`
7. **Update docs** - If you changed functionality, update relevant docs
8. **Commit** - Only if steps 5-6 pass

## Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run all tests
cargo test -p echomancy-core   # Run core tests only
cargo test <name>              # Run tests matching name
cargo clippy                   # Lint all crates
cargo run -p echomancy-bevy    # Run the game
```

## File Locations

| What | Where |
|------|-------|
| Documentation | `docs/` |
| MTG Rules Reference | `docs/reference/MagicCompRules-*.txt` |
| Specifications | `docs/specs/` |
| Game engine core | `crates/echomancy-core/src/domain/` |
| Bevy app | `crates/echomancy-bevy/src/` |
| Specialized agents | `.claude/agents/` |
| Reusable skills | `.claude/skills/` |

## Rust Best Practices Reference

| Guide | URL |
|-------|-----|
| Rust API Guidelines | https://rust-lang.github.io/api-guidelines/checklist.html |
| Effective Rust | https://www.effective-rust.com/ |
| Bevy Best Practices | https://github.com/tbillington/bevy_best_practices |
