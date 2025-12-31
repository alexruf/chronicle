# AGENTS.md

> This file provides development guidelines for AI coding agents (Claude, GitHub Copilot, Cursor, Aider, etc.) working with this codebase. It follows the open [AGENTS.md standard](https://agents.md/).

## Development Philosophy

**Core principles:**
- Incremental progress over big bangs
- Learn from existing code before writing new code
- Pragmatic over dogmatic
- Clear intent over clever code

**Simplicity means:**
- Single responsibility per function/module
- No premature abstractions
- Choose the boring, obvious solution
- If it needs explanation, simplify it

## Development Process

### Before Writing Code

1. **Explore** - Find 3+ similar implementations in the codebase
2. **Understand** - Identify patterns, conventions, and utilities already in use
3. **Plan** - Break complex work into small, testable increments

### Implementation Cycle

1. **Test** - Write failing test first (when practical)
2. **Implement** - Minimal code to pass
3. **Refactor** - Clean up while tests pass
4. **Commit** - Small, working increments with clear messages

### When Stuck

**Stop after 3 failed attempts.** Then:

1. Document what failed and why
2. Research 2-3 alternative approaches
3. Question fundamentals:
   - Wrong abstraction level?
   - Can this be split smaller?
   - Is there a simpler approach?
4. Ask for help or try a completely different angle

## Code Standards

### Architecture

- Composition over inheritance
- Dependency injection over singletons
- Explicit data flow over hidden state
- Test-driven when practical

### Code Quality

**Every commit must:**
- Compile/build successfully
- Pass all tests
- Include tests for new behavior
- Follow project formatting

**Before committing:**
- Run formatter and linter
- Self-review the diff
- Write a message explaining "why", not just "what"

### Error Handling

- Fail fast with descriptive messages
- Include context for debugging
- Handle errors at the appropriate level
- Never silently swallow errors

## Decision Framework

When choosing between approaches, prioritize:

1. **Testability** - Can this be easily tested?
2. **Readability** - Clear in 6 months?
3. **Consistency** - Matches existing patterns?
4. **Simplicity** - Minimal complexity that works?
5. **Reversibility** - Easy to change later?

## Testing Guidelines

- Test behavior, not implementation details
- One concept per test
- Descriptive test names (given/when/then)
- Deterministic - no flaky tests
- Use existing test utilities

## Quality Checklist

Before considering work complete:

- [ ] Tests written and passing
- [ ] Follows project conventions
- [ ] No linter/formatter warnings
- [ ] Clear commit messages
- [ ] No TODOs without tracking (issue/ticket)

## Rules

### Always:
- Commit working code incrementally
- Learn from existing implementations first
- Stop and reassess after 3 failed attempts
- Match the project's existing style and patterns
- Run project's formatter and linter before committing

### Never:
- Use `--no-verify` to bypass commit hooks
- Disable or skip tests instead of fixing them
- Commit code that doesn't build or compile
- Make assumptions - verify against existing code
- Silently swallow errors or exceptions
