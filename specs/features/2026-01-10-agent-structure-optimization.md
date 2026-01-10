# Agent Structure Optimization

**Date**: 2026-01-10
**Status**: Implemented
**Author**: Agent structure analysis and optimization

## Overview

Analyzed the current agent structure for efficiency, identified redundancies and cost optimization opportunities, and implemented a streamlined agent configuration.

## Problem Analysis

### Original Structure (8 agents, 4 Opus)

| Agent | Model | Issue |
|-------|-------|-------|
| agent-orchestrator | Sonnet | Redundant with tech-lead-strategist |
| tech-lead-strategist | Opus | Newly added (good) |
| mtg-product-manager | Opus | ✅ Justified - strategic decisions |
| mtg-spec-writer | Opus | ❌ Over-powered for spec writing |
| senior-backend-engineer | Opus | ❌ Over-powered for implementation |
| ui-engineer | Sonnet | ✅ Correct model |
| typescript-architect | Sonnet | ✅ Correct model |
| mtg-code-reviewer | Sonnet | ✅ Correct model |

### Identified Problems

1. **Redundancy**: agent-orchestrator and tech-lead-strategist had overlapping responsibilities
   - Both did task decomposition
   - Both coordinated multi-agent workflows
   - Unclear when to use which one

2. **Excessive Opus Usage**: 4 agents using Opus without justification
   - Opus is expensive and should be reserved for truly strategic/ambiguous tasks
   - Spec writing and backend implementation are well-defined tasks that Sonnet can handle

3. **Cost Impact**: Unnecessary spending on premium model for routine tasks

## Solution Implemented

### Optimized Structure (7 agents, 1 Opus)

```
🎯 Strategy & Coordination
├─ tech-lead-strategist (Opus)    ← Handles multi-step planning
└─ mtg-product-manager (Opus)     ← Only Opus for product strategy

📝 Documentation
└─ mtg-spec-writer (Sonnet)       ← Changed from Opus

⚙️ Implementation
├─ senior-backend-engineer (Sonnet) ← Changed from Opus
├─ ui-engineer (Sonnet)
└─ typescript-architect (Sonnet)

✅ Quality
└─ mtg-code-reviewer (Sonnet)
```

### Changes Made

1. **Removed agent-orchestrator**
   - Functionality covered by tech-lead-strategist
   - Eliminates decision paralysis
   - Cleaner agent selection logic

2. **Downgraded mtg-spec-writer to Sonnet**
   - Spec writing is technical and well-defined
   - Sonnet is fully capable of structured documentation
   - Significant cost savings

3. **Downgraded senior-backend-engineer to Sonnet**
   - Implementation tasks are well-scoped
   - DDD patterns are established and documented
   - Sonnet handles complex TypeScript/backend code excellently

4. **Updated CLAUDE.md**
   - Removed agent-orchestrator reference
   - Added tech-lead-strategist with clear use case

## Impact

### Cost Savings
- **Before**: 4 agents using Opus (50% of agents)
- **After**: 1 agent using Opus (14% of agents)
- **Reduction**: 75% reduction in Opus usage

### Clarity Improvements
- **Before**: Confusion between agent-orchestrator vs tech-lead-strategist
- **After**: Single clear entry point for multi-step planning
- **Benefit**: Faster decision-making, less overhead

### Coverage Maintained
- All previous capabilities still covered
- No gaps in functionality
- Simpler mental model for agent selection

## Justification for Remaining Opus Usage

**mtg-product-manager (Opus)**: Justified because:
- Makes strategic product decisions with high ambiguity
- Requires balancing business, user experience, and MTG rules
- Handles prioritization across competing objectives
- Benefits from Opus's superior reasoning for open-ended problems

**tech-lead-strategist (Opus)**: Justified because:
- Plans multi-step implementations with architectural implications
- Makes technology selection decisions
- Evaluates trade-offs across time/complexity/maintainability
- Requires deep reasoning about long-term consequences

## Future Considerations

### Potential Further Optimization
If cost becomes a concern, consider:
- Downgrade tech-lead-strategist to Sonnet (test for 1 sprint)
- Monitor quality of strategic planning decisions

### Agent Addition Criteria
Only add new agents if:
1. Clear gap in current coverage
2. Task volume justifies specialization
3. No overlap with existing agents
4. Model selection justified by task complexity

### Do NOT Add
- Testing agent (skills handle this)
- DevOps agent (out of current scope)
- Fullstack agent (use /subagent-driven-development skill)

## Maintenance

This document should be updated when:
- New agents are added or removed
- Model assignments change
- Cost/performance analysis reveals new optimizations
- Agent responsibilities evolve

## Summary

Successfully streamlined agent structure from 8 to 7 agents, reduced Opus usage by 75%, eliminated redundancy, and maintained full functional coverage. The new structure is clearer, more cost-effective, and easier to reason about.
