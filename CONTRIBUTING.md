# Contributing to Echomancy

Thank you for your interest in contributing to **Echomancy**.

Echomancy is an open, transparent Magic rules engine focused on correctness,
explicit modeling, and long-term maintainability.

Before contributing, please read this document carefully.

---

## 📚 Required Reading

Before opening a PR, contributors **must** read:

1. `ROADMAP.md`
2. Engine architecture documentation
3. Existing tests related to the area being modified

The roadmap is the **single source of truth** for project scope.

---

## 🧠 Core Principles

Echomancy follows these non-negotiable principles:

- The engine owns all rules
- The UI never infers rules
- All rule logic is explicit and test-driven
- No hidden shortcuts or heuristic behavior

If a change violates these principles, it will not be merged.

---

## 🧪 Tests Are Mandatory

All rule changes **must** include tests.

- New behavior → new tests
- Bug fix → regression test
- Refactor → no behavior change without discussion

Tests are part of the public contract.

---

## 🚫 What NOT to Do

Please do NOT:
- Add UI concepts to the engine
- Infer rules in the UI layer
- Implement features not listed in the roadmap without discussion
- Introduce card-text parsing or expert systems
- Add “temporary hacks” without explicit TODOs

---

## 🧱 Engine ↔ UI Boundary

The engine:
- Validates all actions
- Owns all state transitions
- Exposes state via `Game.exportState()`

The UI:
- Consumes exported state
- Applies visibility filtering
- Never mutates engine state directly

Breaking this boundary is considered a critical architectural violation.

---

## 📦 Scope of Contributions

Good contributions include:
- Implementing roadmap items
- Improving test coverage
- Clarifying documentation
- Refactoring for clarity (no behavior change)

Large features should be discussed before implementation.

---

## 💬 Communication

When in doubt:
- Open a discussion
- Reference the roadmap
- Explain assumptions explicitly

Clarity beats speed.

---

## 🧭 Final Note

Echomancy is a long-term project.

We value:
- Correctness over completeness
- Transparency over cleverness
- Explicit design over magic

Thank you for helping build a fair and open Magic engine.
