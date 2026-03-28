# Testing Anti-Patterns

**Load this reference when:** writing or changing tests, adding mocks, or tempted to add test-only methods to production code.

## Overview

Tests must verify real behavior, not mock behavior. Mocks are a means to isolate, not the thing being tested.

**Core principle:** Test what the code does, not what the mocks do.

**Following strict TDD prevents these anti-patterns.**

## The Iron Laws

```
1. NEVER test mock behavior
2. NEVER add test-only methods to production structs
3. NEVER mock without understanding dependencies
```

## Anti-Pattern 1: Testing Mock Behavior

**The violation:**
```rust
// BAD: Testing that the mock exists
#[test]
fn renders_sidebar() {
    let mock_sidebar = MockSidebar::new();
    assert!(mock_sidebar.is_visible()); // Testing the mock, not real code!
}
```

**Why this is wrong:**
- You're verifying the mock works, not that the component works
- Test passes when mock is present, fails when it's not
- Tells you nothing about real behavior

**your human partner's correction:** "Are we testing the behavior of a mock?"

**The fix:**
```rust
// GOOD: Test real component behavior
#[test]
fn page_includes_sidebar() {
    let page = Page::new();
    assert!(page.has_sidebar());
}
```

### Gate Function

```
BEFORE asserting on any mock element:
  Ask: "Am I testing real component behavior or just mock existence?"

  IF testing mock existence:
    STOP - Delete the assertion or use the real component

  Test real behavior instead
```

## Anti-Pattern 2: Test-Only Methods in Production

**The violation:**
```rust
// BAD: destroy() only used in tests
impl Session {
    pub fn destroy(&mut self) {  // Looks like production API!
        self.workspace_manager.destroy_workspace(&self.id);
        // ... cleanup
    }
}

// In tests
fn teardown(session: &mut Session) {
    session.destroy();
}
```

**Why this is wrong:**
- Production struct polluted with test-only code
- Dangerous if accidentally called in production
- Violates YAGNI and separation of concerns
- Confuses object lifecycle with entity lifecycle

**The fix:**
```rust
// GOOD: Test utilities handle test cleanup
// Session has no destroy() - it's stateless in production

// In test_helpers module
#[cfg(test)]
pub(crate) fn cleanup_session(session: &Session, manager: &mut WorkspaceManager) {
    if let Some(workspace) = session.get_workspace_info() {
        manager.destroy_workspace(&workspace.id);
    }
}
```

### Gate Function

```
BEFORE adding any method to production struct:
  Ask: "Is this only used by tests?"

  IF yes:
    STOP - Don't add it
    Put it in #[cfg(test)] test utilities instead

  Ask: "Does this struct own this resource's lifecycle?"

  IF no:
    STOP - Wrong struct for this method
```

## Anti-Pattern 3: Mocking Without Understanding

**The violation:**
```rust
// BAD: Mock breaks test logic
#[test]
fn detects_duplicate_server() {
    // Mock prevents config write that test depends on!
    let mock_catalog = MockToolCatalog::new();
    mock_catalog.expect_discover_and_cache_tools()
        .returning(|_| Ok(()));

    add_server(&config).unwrap();
    add_server(&config).unwrap();  // Should error - but won't!
}
```

**Why this is wrong:**
- Mocked method had side effect test depended on (writing config)
- Over-mocking to "be safe" breaks actual behavior
- Test passes for wrong reason or fails mysteriously

**The fix:**
```rust
// GOOD: Mock at correct level
#[test]
fn detects_duplicate_server() {
    // Mock the slow part, preserve behavior test needs
    let mock_manager = MockServerManager::new(); // Just mock slow server startup

    add_server(&config).unwrap();  // Config written
    let result = add_server(&config);  // Duplicate detected
    assert!(result.is_err());
}
```

### Gate Function

```
BEFORE mocking any method:
  STOP - Don't mock yet

  1. Ask: "What side effects does the real method have?"
  2. Ask: "Does this test depend on any of those side effects?"
  3. Ask: "Do I fully understand what this test needs?"

  IF depends on side effects:
    Mock at lower level (the actual slow/external operation)
    OR use test doubles that preserve necessary behavior
    NOT the high-level method the test depends on

  IF unsure what test depends on:
    Run test with real implementation FIRST
    Observe what actually needs to happen
    THEN add minimal mocking at the right level

  Red flags:
    - "I'll mock this to be safe"
    - "This might be slow, better mock it"
    - Mocking without understanding the dependency chain
```

## Anti-Pattern 4: Incomplete Mocks

**The violation:**
```rust
// BAD: Partial mock - only fields you think you need
let mock_response = Response {
    status: Status::Success,
    data: Data { user_id: "123".into(), name: "Alice".into() },
    // Missing: metadata that downstream code uses
    ..Default::default()
};

// Later: panics when code accesses response.metadata.request_id
```

**Why this is wrong:**
- **Partial mocks hide structural assumptions** - You only mocked fields you know about
- **Downstream code may depend on fields you didn't include** - Silent failures
- **Tests pass but integration fails** - Mock incomplete, real API complete
- **False confidence** - Test proves nothing about real behavior

**The Iron Rule:** Mock the COMPLETE data structure as it exists in reality, not just fields your immediate test uses.

**The fix:**
```rust
// GOOD: Mirror real API completeness
let mock_response = Response {
    status: Status::Success,
    data: Data { user_id: "123".into(), name: "Alice".into() },
    metadata: Metadata { request_id: "req-789".into(), timestamp: 1234567890 },
    // All fields real API returns
};
```

### Gate Function

```
BEFORE creating mock responses:
  Check: "What fields does the real struct/API response contain?"

  Actions:
    1. Examine actual struct definition or API response
    2. Include ALL fields system might consume downstream
    3. Verify mock matches real response schema completely

  Critical:
    If you're creating a mock, you must understand the ENTIRE structure
    Partial mocks fail silently when code depends on omitted fields

  If uncertain: Include all documented fields
```

## Anti-Pattern 5: Integration Tests as Afterthought

**The violation:**
```
Implementation complete
No tests written
"Ready for testing"
```

**Why this is wrong:**
- Testing is part of implementation, not optional follow-up
- TDD would have caught this
- Can't claim complete without tests

**The fix:**
```
TDD cycle:
1. Write failing test
2. Implement to pass
3. Refactor
4. THEN claim complete
```

## When Mocks Become Too Complex

**Warning signs:**
- Mock setup longer than test logic
- Mocking everything to make test pass
- Mocks missing methods real components have
- Test breaks when mock changes

**your human partner's question:** "Do we need to be using a mock here?"

**Consider:** Integration tests with real components often simpler than complex mocks

## TDD Prevents These Anti-Patterns

**Why TDD helps:**
1. **Write test first** -> Forces you to think about what you're actually testing
2. **Watch it fail** -> Confirms test tests real behavior, not mocks
3. **Minimal implementation** -> No test-only methods creep in
4. **Real dependencies** -> You see what the test actually needs before mocking

**If you're testing mock behavior, you violated TDD** - you added mocks without watching test fail against real code first.

## Quick Reference

| Anti-Pattern | Fix |
|--------------|-----|
| Assert on mock elements | Test real component or remove mock |
| Test-only methods in production | Move to `#[cfg(test)]` utilities |
| Mock without understanding | Understand dependencies first, mock minimally |
| Incomplete mocks | Mirror real struct/API completely |
| Tests as afterthought | TDD - tests first |
| Over-complex mocks | Consider integration tests |

## Red Flags

- Assertions check mock state instead of real behavior
- Methods only called in test files
- Mock setup is >50% of test
- Test fails when you remove mock
- Can't explain why mock is needed
- Mocking "just to be safe"

## The Bottom Line

**Mocks are tools to isolate, not things to test.**

If TDD reveals you're testing mock behavior, you've gone wrong.

Fix: Test real behavior or question why you're mocking at all.
