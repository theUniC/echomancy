# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Read `AGENTS.md` for all instructions.** It contains the complete guide for working in this codebase, including commands, architecture, coding standards, and workflow.

## Mandatory Implementation Workflow

### 1. Before any implementation

ALWAYS run `/brainstorming` before writing code. No exceptions.

### 2. Use specialized agents

NEVER implement directly. Use:
- `ui-engineer` - React/Next.js components and frontend development
- `senior-backend-engineer` - Backend development using Domain-driven design, TDD, SOLID principles and general best practices
- `mtg-code-reviewer` - Review after implementing and PR review.
- `mtg-product-manager` - To adjust ROADMAP and decide priorities
- `mtg-spec-writer` - To write new feature specs
- `typescript-architect` - To check for Typescript errors and issues
- `tech-lead-strategist` - For multi-step tasks requiring strategic planning and breakdown

For tasks with frontend + backend, use `/subagent-driven-development`.
