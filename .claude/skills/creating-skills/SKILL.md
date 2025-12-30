---
name: creating-skills
description: Use when you need to create a new skill for Claude Code. Defines the structure, best practices, and process for creating reusable skills.
---

# Creating Skills

## Overview

Skills are reusable instruction sets that Claude can invoke via `/skill-name`. They define processes, patterns, and workflows that can be applied across different contexts.

## When to Create a Skill

**Create a skill when:**
- A process is used repeatedly across different tasks
- You need a standardized workflow (e.g., TDD, brainstorming, git workflow)
- The process has multiple steps that are easy to forget
- Multiple agents could benefit from the same instructions

**Don't create a skill when:**
- The process is one-time or highly specific
- An agent already handles this (agents are for roles, skills are for processes)
- It's just a checklist that fits better in `AGENTS.md`

## Skill vs Agent

| Aspect | Skill | Agent |
|--------|-------|-------|
| **Invoked by** | User via `/skill-name` or main Claude | Main Claude via Task tool |
| **Purpose** | Process/workflow | Role/expertise |
| **Context** | Runs in current session | Fresh subprocess |
| **Examples** | TDD, brainstorming, code review process | senior-backend-engineer, ui-engineer |

## File Structure

```
.claude/skills/
└── skill-name/
    └── SKILL.md          # Required: Main skill definition
    └── additional.md     # Optional: Supporting documents
```

## SKILL.md Format

```markdown
---
name: skill-name
description: When to use this skill. Include "(project)" at end if project-specific.
---

# Skill Title

## Overview
Brief description of what this skill does and its core principle.

## Project Context (if project-specific)
- What files to read first
- Project-specific patterns to follow
- Commands to use (bun vs npm, etc.)

## When to Use
- Bullet points of situations where this skill applies

## Process
### Step 1: ...
### Step 2: ...
(Clear, actionable steps)

## Verification
Checklist or criteria to confirm skill was applied correctly.

## Red Flags / Anti-Patterns (optional)
What to avoid when using this skill.
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Kebab-case identifier (e.g., `test-driven-development`) |
| `description` | Yes | When to use. Add `(project)` suffix if project-specific |

## Best Practices

### Content Guidelines

1. **Be actionable** - Each step should be something Claude can do
2. **Be specific** - Avoid vague instructions like "ensure quality"
3. **Include examples** - Show correct and incorrect approaches
4. **Add verification** - How to know the skill was applied correctly
5. **Reference project context** - Point to `AGENTS.md`, `docs/`, etc.

### Project Integration

For Echomancy skills, always include:
```markdown
## Project Context

Before starting, read:
- `AGENTS.md` - Project rules and workflow
- Relevant files in `docs/`

Use project commands:
- `bun test` (not npm test)
- `bun run lint && bun run format`
```

### Avoid

- Implementation details that belong in code
- References to non-existent skills or files
- Generic content that doesn't add value
- Overly long skills (break into multiple if needed)

## Assigning Skills to Agents

Add skills to agent frontmatter:

```yaml
---
name: agent-name
description: ...
model: sonnet
color: blue
skills: skill-one,skill-two
---
```

Also add a "Related Skills" section in the agent body:

```markdown
## Related Skills

When working on tasks, apply these skills:
- **`/skill-one`** - Brief description
- **`/skill-two`** - Brief description
```

## Verification Checklist

Before committing a new skill:
- [ ] Name is kebab-case and descriptive
- [ ] Description clearly states when to use
- [ ] Has Project Context section (if project-specific)
- [ ] Steps are actionable and specific
- [ ] Includes verification criteria
- [ ] No broken references to other skills/files
- [ ] Uses correct project commands (bun, not npm)
- [ ] Assigned to relevant agents (if applicable)

## Example: Creating a Git Workflow Skill

```markdown
---
name: git-workflow
description: Use to finalize work with verification, commit, and optional PR creation (project)
---

# Git Workflow

## Project Context
Run before committing: `bun test && bun run lint && bun run format`

## Process

### 1. Verify
- All tests pass
- Linting passes
- Formatting applied

### 2. Commit
- Stage relevant files
- Write descriptive commit message
- Follow conventional commits if project uses them

### 3. PR (if requested)
- Push branch
- Create PR with summary

## Verification
- [ ] Tests green
- [ ] Lint clean
- [ ] Commit made
```
