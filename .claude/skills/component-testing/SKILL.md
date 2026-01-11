---
name: component-testing
description: Use when testing React components in Next.js applications. Covers visual, integration, and accessibility testing patterns specific to UI. (project)
---

# Component Testing

Testing patterns for React components that differ from traditional TDD. Focus on user behavior, accessibility, and visual correctness.

**Core principle:** Test what the user sees and does, not implementation details.

## Project Context

**Test runner:** Vitest with React Testing Library

**Run tests:**
```bash
bun test                    # All tests
bun test <component-name>   # Specific component
```

**Key files:**
- `src/echomancy/infrastructure/ui/` - UI components
- `docs/specs/features/ui-mvp.md` - UI specification

## When to Use

- Creating new React components
- Adding interactivity to existing components
- Fixing UI bugs
- Ensuring accessibility compliance

## Testing Philosophy

### Test User Behavior, Not Implementation

```typescript
// ❌ BAD: Testing implementation
expect(component.state.isOpen).toBe(true)
expect(wrapper.find('div.dropdown')).toHaveLength(1)

// ✅ GOOD: Testing user behavior
expect(screen.getByRole('menu')).toBeVisible()
await user.click(screen.getByRole('button', { name: /open menu/i }))
expect(screen.getByRole('menuitem', { name: /settings/i })).toBeInTheDocument()
```

### Query Priority

Use queries in this order (most to least preferred):

| Priority | Query | When to use |
|----------|-------|-------------|
| 1 | `getByRole` | Interactive elements (buttons, links, inputs) |
| 2 | `getByLabelText` | Form fields |
| 3 | `getByPlaceholderText` | Inputs without labels |
| 4 | `getByText` | Non-interactive text content |
| 5 | `getByTestId` | Last resort when nothing else works |

## Testing Patterns

### 1. Render and Assert

Basic component rendering:

```typescript
import { render, screen } from '@testing-library/react'
import { CardDisplay } from './CardDisplay'

test('displays card name and mana cost', () => {
  const card = { name: 'Lightning Bolt', manaCost: '{R}' }

  render(<CardDisplay card={card} />)

  expect(screen.getByText('Lightning Bolt')).toBeInTheDocument()
  expect(screen.getByText('{R}')).toBeInTheDocument()
})
```

### 2. User Interactions

Test clicks, typing, and other user actions:

```typescript
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ManaPool } from './ManaPool'

test('tapping land adds mana to pool', async () => {
  const user = userEvent.setup()
  const onManaAdd = vi.fn()

  render(<ManaPool onManaAdd={onManaAdd} />)

  await user.click(screen.getByRole('button', { name: /tap forest/i }))

  expect(onManaAdd).toHaveBeenCalledWith({ color: 'green', amount: 1 })
})
```

### 3. Async Operations

Test loading states and async updates:

```typescript
test('shows loading then displays game state', async () => {
  render(<GameBoard gameId="123" />)

  // Loading state
  expect(screen.getByText(/loading/i)).toBeInTheDocument()

  // Wait for content
  await waitFor(() => {
    expect(screen.getByRole('region', { name: /battlefield/i })).toBeInTheDocument()
  })

  expect(screen.queryByText(/loading/i)).not.toBeInTheDocument()
})
```

### 4. Error States

Test error handling:

```typescript
test('displays error message when game fails to load', async () => {
  server.use(
    http.get('/api/game/:id', () => {
      return HttpResponse.json({ error: 'Game not found' }, { status: 404 })
    })
  )

  render(<GameBoard gameId="invalid" />)

  await waitFor(() => {
    expect(screen.getByRole('alert')).toHaveTextContent(/game not found/i)
  })
})
```

### 5. Accessibility Testing

Ensure components are accessible:

```typescript
import { axe, toHaveNoViolations } from 'jest-axe'

expect.extend(toHaveNoViolations)

test('card component has no accessibility violations', async () => {
  const { container } = render(<CardDisplay card={mockCard} />)

  const results = await axe(container)

  expect(results).toHaveNoViolations()
})
```

**Manual accessibility checks:**
- [ ] All interactive elements focusable with keyboard
- [ ] Focus order makes sense
- [ ] Color contrast meets WCAG AA
- [ ] Images have alt text
- [ ] Form inputs have labels

## What NOT to Test

- Styling details (colors, sizes) - use visual regression instead
- Third-party library internals
- Implementation details (state, refs, internal methods)
- Things already tested by the library

## Test File Structure

```typescript
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ComponentName } from './ComponentName'

describe('ComponentName', () => {
  // Setup shared across tests
  const defaultProps = { /* ... */ }

  describe('rendering', () => {
    test('displays required elements', () => { /* ... */ })
    test('handles empty state', () => { /* ... */ })
  })

  describe('user interactions', () => {
    test('responds to click', async () => { /* ... */ })
    test('handles keyboard navigation', async () => { /* ... */ })
  })

  describe('accessibility', () => {
    test('has no axe violations', async () => { /* ... */ })
  })
})
```

## Game UI Specific Patterns

### Testing Game State Display

```typescript
test('displays current phase indicator', () => {
  const snapshot: GameSnapshot = {
    currentPhase: 'combat',
    activePlayer: 'player1',
    // ...
  }

  render(<PhaseIndicator snapshot={snapshot} />)

  expect(screen.getByRole('status')).toHaveTextContent(/combat phase/i)
})
```

### Testing Card Interactions

```typescript
test('hovering card shows preview', async () => {
  const user = userEvent.setup()

  render(<HandDisplay cards={mockCards} />)

  await user.hover(screen.getByRole('button', { name: /lightning bolt/i }))

  expect(screen.getByRole('tooltip')).toBeVisible()
})
```

### Testing Drag and Drop

```typescript
test('can drag card from hand to battlefield', async () => {
  const onPlay = vi.fn()

  render(<GameBoard onCardPlay={onPlay} />)

  const card = screen.getByRole('button', { name: /forest/i })
  const battlefield = screen.getByRole('region', { name: /battlefield/i })

  await drag(card).to(battlefield)

  expect(onPlay).toHaveBeenCalledWith(expect.objectContaining({ name: 'Forest' }))
})
```

## Verification Checklist

Before completing component tests:
- [ ] Tested all user-visible states (loading, empty, error, success)
- [ ] Tested all user interactions (clicks, keyboard, hover if relevant)
- [ ] Used accessible queries (`getByRole`, `getByLabelText`)
- [ ] Tested edge cases (empty data, long text, many items)
- [ ] Accessibility check passes
- [ ] No implementation details tested

## Red Flags

**Stop and reconsider if:**
- Testing component state directly
- Using `getByTestId` as first choice
- Mocking child components excessively
- Tests break when refactoring (but behavior unchanged)
- Snapshot tests without clear purpose
