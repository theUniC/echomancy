# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Read `AGENTS.md` for all instructions.** It contains the complete guide for working in this codebase, including commands, architecture, coding standards, and workflow.

---

## MANDATORY: Complete Development Workflow

### Decision Tree: What to Do First?

```
┌─────────────────────────────────────────────────┐
│ USER REQUEST                                    │
└─────────────────────────────────────────────────┘
                    ↓
         New feature or priority change?
                    ↓
        ┌───────────┴───────────┐
       YES                      NO
        ↓                        ↓
   mtg-product-manager    Is it trivial work?
        ↓                        ↓
   Decide priority          ┌───┴────┐
   Update ROADMAP          YES       NO
        ↓                   ↓          ↓
   Needs spec?          Implement   tech-lead-strategist
        ↓                directly       ↓
    ┌───┴────┐           (typo,      Plans &
   YES      NO           format)     coordinates
    ↓        ↓                          ↓
mtg-spec-  Continue                Specialized
writer                             agents
```

---

## Phase 1: Strategic Planning (Product)

**WHEN**: New feature, scope change, prioritization needed

**WHO**: `mtg-product-manager`

**RESPONSIBILITIES**:
- Decide WHAT to build and WHEN
- Update ROADMAP with priorities
- Validate features against MTG rules and player expectations
- Call `mtg-spec-writer` if new spec needed

**OUTPUT**:
- Updated ROADMAP
- Decision: implement now / later / never
- New spec in `specs/backlog/` (if needed)

---

## Phase 2: Technical Planning (Implementation)

**WHEN**: Feature approved and spec in `specs/active/`

**WHO**: `tech-lead-strategist`

**RESPONSIBILITIES**:
- Analyze spec thoroughly
- Decompose into implementation tasks
- Decide which specialist agents to use
- Coordinate workflow (sequential or parallel)
- Identify technical risks and dependencies

**OUTPUT**: Implementation plan with agent assignments

---

## Phase 3: Implementation (Specialized Work)

**WHEN**: After tech-lead planning

**WHO**: Specialist agents (NEVER implement directly)

**Available Specialists**:
- `tcg-ui-designer` - Visual design (layout, states, aesthetics) - **use BEFORE ui-engineer**
- `ui-engineer` - React/Next.js frontend implementation
- `senior-backend-engineer` - Backend API, domain logic, DDD patterns
- `typescript-architect` - Complex type system issues
- `/subagent-driven-development` - Parallel frontend + backend work

**COORDINATED BY**: `tech-lead-strategist`

**KEY RULE**: ALL implementation goes through specialized agents. No exceptions.

**IMPORTANT**: For UI features, visual design (`tcg-ui-designer`) should happen BEFORE implementation (`ui-engineer`).

---

## Phase 4: Quality Assurance

**MANDATORY** for all non-trivial work:

1. **Tests**: `bun test` - All tests must pass
2. **Linting**: `bun run lint && bun run format` - Code style compliance
3. **Code Review**: `mtg-code-reviewer` - Quality and MTG rules verification
4. **Fix Issues**: Address any problems and repeat QA

---

## Phase 5: Finalization

1. **Update Documentation**: If architectural changes, update `docs/`
2. **Move Spec**: `specs/active/` → `specs/done/`
3. **Commit**: Use `/git-workflow` skill

---

## Key Principles

### ⚡ NEVER Implement Directly
ALL work goes through specialized agents. No exceptions.

### 🎯 Tech Lead Coordinates Implementation
For any non-trivial work, `tech-lead-strategist`:
- Owns the implementation plan
- Coordinates specialist agents
- Ensures quality gates are met
- Breaks down complex tasks

### 📋 Product Manager Owns Priorities
`mtg-product-manager`:
- Decides WHAT to build
- Decides WHEN to build it
- Updates ROADMAP
- Can create specs via `mtg-spec-writer`

### 🔧 Specialists Execute
Implementation agents execute plans, never create strategy.

---

## Exceptions: Trivial Work (Skip Tech Lead)

You MAY implement directly for:
- ✅ Typo fixes
- ✅ Formatting/linting changes
- ✅ Moving files
- ✅ Documentation updates (non-architectural)
- ✅ Single-line bug fixes

**Everything else requires `tech-lead-strategist`**

---

## Agent Quick Reference

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| `mtg-product-manager` | Product strategy, ROADMAP | New features, prioritization |
| `mtg-spec-writer` | Write specifications | Need detailed spec document |
| `tech-lead-strategist` | Plan implementation | Any non-trivial work |
| `tcg-ui-designer` | Visual design for TCG UI | Layout, visual states, aesthetics |
| `ui-engineer` | Frontend implementation | React/Next.js work |
| `senior-backend-engineer` | Backend implementation | API, domain logic, DDD |
| `typescript-architect` | Type system issues | Complex TypeScript problems |
| `mtg-code-reviewer` | Code review | After implementation |

**Skills**:
- `/subagent-driven-development` - Parallel frontend + backend work
- `/brainstorming` - Explore ideas before specs
- `/git-workflow` - Commit and PR creation
