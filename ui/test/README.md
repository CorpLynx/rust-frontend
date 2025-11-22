# Testing Infrastructure

This directory contains the testing infrastructure for the Prometheus Tauri UI.

## Overview

The testing setup uses:
- **Vitest** - Fast unit test framework with jsdom environment
- **fast-check** - Property-based testing library for generating test cases
- **@testing-library/dom** - DOM testing utilities for querying and interacting with elements

## Configuration

### vitest.config.js
- Environment: jsdom (simulates browser DOM)
- Globals: enabled (no need to import describe/it/expect)
- Setup file: `./test/setup.js` (runs before each test file)
- Test pattern: `**/*.test.js`

### Test Scripts

```bash
npm test          # Run all tests once
npm run test:watch # Run tests in watch mode
npm run test:ui   # Run tests with UI dashboard
```

## Test Utilities (test/utils.js)

### DOM Manipulation
- `loadHTML(html)` - Load HTML string into document.body
- `loadMainUI()` - Load the complete UI structure for testing

### Event Simulation
- `click(element)` - Simulate click event
- `hover(element)` - Simulate hover event
- `typeText(input, text)` - Simulate typing in input field

### Animation & Timing
- `waitForAnimation(duration)` - Wait for animation to complete
- `waitFor(callback)` - Wait for condition (from @testing-library/dom)

### Style & Visibility
- `getStyle(element, property)` - Get computed CSS property value
- `isVisible(element)` - Check if element is visible

### Window & Layout
- `resizeWindow(width, height)` - Simulate window resize

### Mocking
- `mockTauriCommand(command, handler)` - Mock Tauri backend commands
- `createMockConversation(overrides)` - Create mock conversation object
- `createMockMessage(role, content)` - Create mock message object

### Testing Library Re-exports
- `screen` - Query elements in the document
- `waitFor` - Wait for async conditions

## Property-Based Testing

Property-based tests use fast-check to generate random inputs and verify properties hold across all cases.

### Example Property Test

```javascript
import * as fc from 'fast-check';

// Feature: tauri-ui-improvements, Property 1: Sidebar toggle behavior
test('sidebar toggles between open and closed states', () => {
  fc.assert(
    fc.property(fc.boolean(), (initialState) => {
      // Setup
      loadMainUI();
      const sidebar = document.querySelector('.sidebar');
      sidebar.dataset.state = initialState ? 'open' : 'closed';
      
      // Action
      const hamburger = document.getElementById('hamburger-btn');
      click(hamburger);
      
      // Verify
      const expectedState = initialState ? 'closed' : 'open';
      return sidebar.dataset.state === expectedState;
    }),
    { numRuns: 100 }
  );
});
```

### Property Test Requirements

1. Each property test must run minimum 100 iterations: `{ numRuns: 100 }`
2. Each test must be tagged with a comment referencing the design document:
   ```javascript
   // Feature: tauri-ui-improvements, Property X: Property description
   ```
3. Each property test must validate a single correctness property from the design

## Unit Testing

Unit tests verify specific examples and edge cases.

### Example Unit Test

```javascript
// Example 1: Hamburger icon presence on load
test('hamburger icon exists on load', () => {
  loadMainUI();
  const hamburger = screen.getByRole('button', { name: /toggle conversations menu/i });
  expect(hamburger).toBeDefined();
  expect(hamburger.querySelector('.hamburger-icon')).toBeDefined();
});
```

## Test Organization

- `setup.js` - Global test setup and Tauri API mocking
- `utils.js` - Reusable test utilities
- `infrastructure.test.js` - Verification that testing setup works correctly
- `*.test.js` - Feature-specific test files

## Running Tests

Before running tests, ensure dependencies are installed:

```bash
cd ui
npm install
```

Then run tests:

```bash
npm test                    # Run all tests once
npm run test:watch          # Watch mode for development
npm run test:ui             # Interactive UI dashboard
```

## Debugging Tests

1. Use `console.log()` to inspect values during test execution
2. Use `screen.debug()` to print current DOM state
3. Use `test.only()` to run a single test
4. Use `describe.only()` to run a single test suite
5. Check `document.body.innerHTML` to see current DOM state

## Coverage

Coverage reports are generated in the `coverage/` directory when running tests with coverage enabled:

```bash
npm test -- --coverage
```
