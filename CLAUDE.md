# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Read `AGENTS.md` for all instructions.** It contains the complete guide for working in this codebase, including commands, architecture, coding standards, and workflow.

---

## MANDATORY: Complete Development Workflow

### First and most important rule

**MANDATORY** - Never ever in any single case you're allowed to skip the decision tree below. So don't jump to implement or do the work yourself.

### Decision Tree: What to Do First?

```
┌─────────────────────────────────────────────────┐
│ USER REQUEST                                    │
└─────────────────────────────────────────────────┘
                    ↓
            Is it trivial work?
            (typo, format, move file)
                    ↓
        ┌───────────┴───────────┐
       YES                      NO
        ↓                        ↓
    Implement              Needs a spec?
    directly                    ↓
                        ┌───────┴───────┐
                       YES              NO
                        ↓                ↓
                 mtg-spec-writer   tech-lead-strategist
                        ↓                ↓
                 (validate with    Plans &
                  mtg-domain-      coordinates
                  expert if                ↓
                  rules-heavy)     Is it UI work?
                                          ↓
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

## Phase 1: Specification (When Needed)

**WHEN**: New feature that needs detailed requirements

**WHO**: `mtg-spec-writer`

**RESPONSIBILITIES**:
- Write clear, detailed specifications
- Focus on WHAT and WHY, not HOW
- Define acceptance criteria
- Document edge cases

**OUTPUT**: New spec in `docs/specs/backlog/`

**VALIDATION**: For rules-heavy features, use `mtg-domain-expert` to validate the spec covers all MTG rules correctly.

---

## Phase 1.5: Rules Completeness Audit (Optional)

**WHEN**: Before implementing rules-heavy features, or when suspecting gaps

**WHO**: `mtg-domain-expert`

**RESPONSIBILITIES**:
- Validate specs against MTG comprehensive rules
- Identify missing dependencies
- Flag impossible or incomplete features

**OUTPUT**:
- Gap analysis report
- List of missing dependencies
- Recommendations for what needs to be added to spec

---

## Phase 2: Technical Planning (Implementation)

**WHEN**: Spec ready in `docs/specs/active/`

**WHO**: `tech-lead-strategist`

**RESPONSIBILITIES**:
- Analyze spec thoroughly
- Decompose into implementation tasks
- Decide which specialist agents to use
- Coordinate workflow (sequential or parallel)
- Identify technical risks and dependencies

**OUTPUT**: Implementation plan with agent assignments

**CRITICAL**: `tech-lead-strategist` MUST add an "Implementation Tracking" section to the end of the active spec file.

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

---

## Phase 3: Implementation (Specialized Work)

**WHEN**: After tech-lead planning

**WHO**: Specialist agents (NEVER implement directly)

**Available Specialists**:
- `tcg-ui-designer` - Visual design (layout, states, aesthetics)
- `ui-engineer` - React/Next.js frontend implementation
- `senior-backend-engineer` - Backend API, domain logic, DDD patterns
- `/subagent-driven-development` - Parallel frontend + backend work

**COORDINATED BY**: `tech-lead-strategist`

**KEY RULE**: ALL implementation goes through specialized agents. No exceptions.

**UI Design Guidelines**:
- **New layouts/components**: Use `tcg-ui-designer` first, then `ui-engineer`
- **Small UI changes** (tweaks, bug fixes, minor adjustments): `ui-engineer` directly

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

### NEVER Implement Directly
ALL work goes through specialized agents. No exceptions.

### Tech Lead Coordinates Implementation
For any non-trivial work, `tech-lead-strategist`:
- Owns the implementation plan
- Coordinates specialist agents
- Ensures quality gates are met
- Breaks down complex tasks

### Specialists Execute
Implementation agents execute plans, never create strategy.

---

## Exceptions: Trivial Work (Skip Tech Lead)

You MAY implement directly for:
- Typo fixes
- Formatting/linting changes
- Moving files
- Documentation updates (non-architectural)
- Single-line bug fixes

**Everything else requires `tech-lead-strategist`**

---

## Agent Quick Reference

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| `mtg-domain-expert` | MTG rules validation | Validate specs for rules completeness |
| `mtg-spec-writer` | Write specifications | Need detailed spec document |
| `tech-lead-strategist` | Plan implementation | Any non-trivial work |
| `tcg-ui-designer` | Visual design for TCG UI | New layouts, visual states, aesthetics |
| `ui-engineer` | Frontend implementation | React/Next.js work |
| `senior-backend-engineer` | Backend implementation | API, domain logic, DDD |
| `mtg-code-reviewer` | Code review | After implementation |

**Skills**:
- `/subagent-driven-development` - Parallel frontend + backend work
- `/brainstorming` - Explore ideas before specs
- `/git-workflow` - Commit and PR creation
