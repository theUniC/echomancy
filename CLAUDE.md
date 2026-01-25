# CLAUDE.md

Read `AGENTS.md` first for commands, file locations, and critical rules.

This file defines the agent workflow for Claude Code.

## Decision Tree (MANDATORY)

Never skip this. For every request:

1. **Trivial?** (typo, format, single-line fix) → Implement directly, go to Phase 4
2. **Needs spec?** (new feature) → `mtg-spec-writer` → (if rules-heavy: `mtg-domain-expert`)
3. **Plan** → `tech-lead-strategist`
4. **Implement**:
   - UI with visual design needs → `tcg-ui-designer` → `ui-engineer`
   - UI without design needs → `ui-engineer`
   - Backend → `senior-backend-engineer`
5. **QA** → `bun run test` → `bun run lint` → `mtg-code-reviewer` → `qa-validator`
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
**Agents**: `ui-engineer`, `senior-backend-engineer`, `tcg-ui-designer`
**Rule**: ALL implementation through specialized agents. No exceptions.

### Phase 4: QA
**Required for all non-trivial work**:
1. `bun run test` - All tests pass
2. `bun run lint && bun run format` - Style compliance
3. `mtg-code-reviewer` - Code quality
4. `qa-validator` - Verify ALL acceptance criteria, mark `[x]`

### Phase 5: Finalization
**Prerequisite**: `qa-validator` passed
1. Update `docs/` if architectural changes
2. Move spec `active/` → `done/`
3. Use `/git-workflow`

## Agent Reference

| Agent | Purpose |
|-------|---------|
| `mtg-spec-writer` | Write specifications |
| `mtg-domain-expert` | Validate MTG rules completeness |
| `tech-lead-strategist` | Plan implementation + QA |
| `tcg-ui-designer` | Visual design (layout, states) |
| `ui-engineer` | React/Next.js implementation |
| `senior-backend-engineer` | Backend, domain logic, DDD |
| `mtg-code-reviewer` | Code review |
| `qa-validator` | Verify acceptance criteria (MANDATORY before done) |

**Skills**: `/subagent-driven-development`, `/brainstorming`, `/git-workflow`
