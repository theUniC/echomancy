---
name: git-workflow
description: Use to finalize work with verification, commit, and optional PR creation. Ensures all quality gates pass before committing. (project)
---

# Git Workflow

Finalize work by running verification, committing changes, and optionally creating a PR.

**Core principle:** Never commit without passing all quality gates first.

## Project Context

**Required commands before commit:**
```bash
bun test && bun run lint && bun run format
```

**Check if docs need updating:**
- New functionality → Update relevant `docs/*.md`
- New specs → Save to `docs/specs/`
- Changed behavior → Update affected documentation

## Process

### 1. Verify Quality Gates

Run all checks:

```bash
bun test && bun run lint && bun run format
```

**All three must pass.** If any fails:
- Fix the issue
- Re-run verification
- Do not proceed until green

### 2. Check Documentation

Ask yourself:
- Did I add new functionality? → Update `docs/`
- Did I change existing behavior? → Update affected docs
- Did I add a new feature? → Consider adding to `docs/specs/features/`

### 3. Review Changes

```bash
git status
git diff
```

Verify:
- No unintended files staged
- No secrets or credentials (`.env`, API keys)
- Changes match what was implemented

### 4. Commit

**Stage relevant files:**
```bash
git add <files>
```

**Write commit message:**
```bash
git commit -m "$(cat <<'EOF'
<type>: <short description>

<optional body explaining why>
EOF
)"
```

**Commit types:**
| Type | When to use |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code change that neither fixes nor adds |
| `test` | Adding or updating tests |
| `docs` | Documentation only |
| `chore` | Maintenance, dependencies, config |

### 5. PR Creation (if requested)

**Push branch:**
```bash
git push -u origin <branch-name>
```

**Create PR:**
```bash
gh pr create --title "<title>" --body "$(cat <<'EOF'
## Summary
<1-3 bullet points describing the change>

## Test plan
- [ ] Tests pass locally
- [ ] Manual verification of <specific behavior>
EOF
)"
```

## Quick Reference

```bash
# Full workflow
bun test && bun run lint && bun run format
git add <files>
git commit -m "<type>: <description>"

# With PR
git push -u origin <branch>
gh pr create --title "<title>" --body "<body>"
```

## Red Flags

**Stop and fix if:**
- Tests failing
- Lint errors
- Committing `.env` or credentials
- Large unrelated changes in diff
- Missing documentation for new features

**Never:**
- Skip verification to "save time"
- Commit with failing tests
- Force push to main/master
- Commit secrets (even "temporarily")

## Verification Checklist

Before marking work complete:
- [ ] `bun test` passes
- [ ] `bun run lint` passes
- [ ] `bun run format` applied
- [ ] Documentation updated (if applicable)
- [ ] Commit message follows convention
- [ ] No secrets in committed files
- [ ] PR created (if requested)
