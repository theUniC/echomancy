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
mtg-spec-  Continue                Is it UI work?
writer                                  ↓
                                ┌───────┴────────┐
                               YES              NO
                                ↓                ↓
                         Has visual      Backend/TS
                         design needs?   specialist
                                ↓
                         ┌──────┴──────┐
                        YES            NO
                         ↓              ↓
                    tcg-ui-designer  ui-engineer
                         ↓           (directly)
                    ui-engineer
                    (implements
                     design)
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
- New spec in `docs/specs/backlog/` (if needed)

**FOLLOW-UP**: After ROADMAP updates, use `mtg-domain-expert` to validate completeness

---

## Phase 1.5: Rules Completeness Audit (Optional but Recommended)

**WHEN**: After ROADMAP changes, before major development phases, or when suspecting gaps

**WHO**: `mtg-domain-expert`

**RESPONSIBILITIES**:
- Audit ROADMAP for logical gaps and missing dependencies
- Validate features can work according to MTG comprehensive rules
- Identify assumptions about unbuilt systems
- Flag impossible or incomplete features

**OUTPUT**:
- Gap analysis report
- List of missing dependencies
- Recommendations for what needs to be built (not when)

**IMPORTANT**: Domain expert does NOT make product decisions. It only validates completeness. PM decides what to do with findings.

---

## Phase 2: Technical Planning (Implementation)

**WHEN**: Feature approved and spec in `docs/specs/active/`

**WHO**: `tech-lead-strategist`

**RESPONSIBILITIES**:
- Analyze spec thoroughly
- Decompose into implementation tasks
- Decide which specialist agents to use
- Coordinate workflow (sequential or parallel)
- Identify technical risks and dependencies

**OUTPUT**: Implementation plan with agent assignments

**CRITICAL**: `tech-lead-strategist` MUST add an "Implementation Tracking" section to the end of the active spec file. This enables:
- Recovery after interruptions
- Progress visibility
- Complete implementation history in completed specs

---

## Implementation Tracking in Specs

All specs in `docs/specs/active/` contain an "Implementation Tracking" section at the end.

### Structure
```markdown
## Implementation Tracking

**Status**: Not Started | In Progress | Completed
**Started**: YYYY-MM-DD
**Completed**: YYYY-MM-DD
**Agent**: {agent-name}

### Task Breakdown

#### Phase 1: {Name} ⏳/🔄/✅
- [ ] Task 1
- [x] Task 2 (completed)

**Blockers**: None | {description}
**Notes**: {context}
```

### Emoji Legend
- ⏳ = Pending (not started)
- 🔄 = In Progress
- ✅ = Completed

### Responsibilities

**tech-lead-strategist**: Creates this section when planning
**Implementation agents**: Update checkboxes, emojis, and dates as they work
**Everyone**: Can see current progress by reading the active spec

### Why This Matters
- **Resume work**: Easy to pick up after interruptions
- **Visibility**: Anyone can check progress without asking
- **History**: Completed specs in `done/` show full implementation journey

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
2. **Move Spec**: `docs/specs/active/` → `docs/specs/done/`
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
| `mtg-domain-expert` | MTG rules validation | Audit ROADMAP/specs for completeness |
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
