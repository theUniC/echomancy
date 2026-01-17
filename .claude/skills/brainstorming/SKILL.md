---
name: brainstorming
description: "Use before designing new features to explore ideas collaboratively. Not needed for bugfixes or small changes. (project)"
---

# Brainstorming Ideas Into Designs

## Overview

Help turn ideas into fully formed designs and specs through natural collaborative dialogue.

Start by understanding the current project context, then ask questions one at a time to refine the idea. Once you understand what you're building, present the design in small sections (200-300 words), checking after each section whether it looks right so far.

## Project Context

Before brainstorming, read:
- `AGENTS.md` - Project rules and workflow
- `docs/` - Architecture, ability system, effect system, etc.
- Relevant existing code to understand current patterns

**Echomancy is a TCG engine.** Consider:
- Game state mutations via `game.apply()`
- Effect system for card abilities
- Stack and priority for spell resolution
- Turn structure and phases

## The Process

**Understanding the idea:**
- Check out the current project state first (files, docs, recent commits)
- Ask questions one at a time to refine the idea
- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message - if a topic needs more exploration, break it into multiple questions
- Focus on understanding: purpose, constraints, success criteria

**Exploring approaches:**
- Propose 2-3 different approaches with trade-offs
- Present options conversationally with your recommendation and reasoning
- Lead with your recommended option and explain why

**Presenting the design:**
- Once you believe you understand what you're building, present the design
- Break it into sections of 200-300 words
- Ask after each section whether it looks right so far
- Cover: architecture, components, data flow, error handling, testing
- Be ready to go back and clarify if something doesn't make sense

## After the Design

**Documentation:**
- Write the validated design to `docs/specs/backlog/NN-<topic>.md` (check existing files for next number)
- Follow the spec structure: Overview, User Stories, Player Experience, Game Rules, Acceptance Criteria, Out of Scope
- Commit the design document to git

**Implementation (if continuing):**
- Ask: "Ready to implement?"
- Create a plan using TodoWrite with concrete steps
- Use `/subagent-driven-development` for multi-step implementations
- Use `/test-driven-development` for each implementation step

## Key Principles

- **One question at a time** - Don't overwhelm with multiple questions
- **Multiple choice preferred** - Easier to answer than open-ended when possible
- **YAGNI ruthlessly** - Remove unnecessary features from all designs
- **Explore alternatives** - Always propose 2-3 approaches before settling
- **Incremental validation** - Present design in sections, validate each
- **Be flexible** - Go back and clarify when something doesn't make sense