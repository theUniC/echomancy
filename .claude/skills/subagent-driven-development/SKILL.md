---
name: subagent-driven-development
description: Use when facing 2+ independent tasks that can be worked on without shared state or sequential dependencies - dispatches fresh subagent for each task (sequential or parallel) with code review between tasks, enabling fast iteration with quality gates (project)
---

# Subagent-Driven Development

Execute multiple tasks by dispatching fresh subagents, with code review between tasks.

**Core principle:** Fresh subagent per task + review after = high quality, fast iteration.

## When to Use

- **2+ independent tasks** that don't share state
- **Parallel work possible** (e.g., UI + backend simultaneously)
- **Sequential tasks** where each builds on the previous

**Don't use when:**
- Single task
- Tasks heavily depend on each other's output
- Exploratory work where scope is unclear

## Process

### 1. Identify Tasks

Break work into independent units:
- Each task should be completable by one agent
- Define clear inputs and outputs
- Identify dependencies between tasks

### 2. Decide: Sequential or Parallel?

**Sequential** (one at a time):
- Tasks depend on each other
- Same files being modified
- Need to see results before next step

**Parallel** (simultaneous):
- Tasks are independent (different files/subsystems)
- No shared state
- Can be reviewed together at the end

### 3. Dispatch Subagents

For each task, dispatch a fresh agent:

```
Task tool:
  subagent_type: [ui-engineer | senior-backend-engineer | general-purpose]
  description: "Implement [task name]"
  prompt: |
    Task: [clear description]

    Requirements:
    - [specific requirement 1]
    - [specific requirement 2]

    Constraints:
    - Read AGENTS.md for project rules
    - Use bun test (not npm)
    - Follow TDD

    Report back: What you implemented, test results, files changed
```

### 4. Review After Each Task (or Batch)

After subagent completes:

```
Task tool:
  subagent_type: mtg-code-reviewer
  description: "Review [task name] implementation"
  prompt: |
    Review changes for [task name].

    Check:
    1. Code quality
    2. Test coverage
    3. AGENTS.md compliance

    Return: Issues (Critical/Important/Minor), Assessment
```

**If Critical issues:** Fix before proceeding
**If Important issues:** Fix before next task
**If Minor issues:** Note for later

### 5. Finalize

After all tasks complete:
- Run full test suite: `bun test`
- Use `/git-workflow` to commit

## Example: UI + Backend in Parallel

```
Task 1 (parallel): ui-engineer
  "Implement HandDisplay component showing cards in player's hand"

Task 2 (parallel): senior-backend-engineer
  "Add getHand query to return player's hand from GameSnapshot"

[Both complete]

Task 3: mtg-code-reviewer
  "Review HandDisplay and getHand implementations"

[Fix any issues]

/git-workflow
```

## Red Flags

**Stop if:**
- Subagent is stuck or confused → provide more context
- Tasks are conflicting → make sequential instead
- Scope is growing → pause and re-plan

**Never:**
- Skip code review
- Proceed with Critical issues unfixed
- Let parallel agents edit same files
