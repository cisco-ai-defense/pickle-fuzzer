# How to Contribute

Thanks for your interest in contributing to `pickle-whip`! Here are a few
general guidelines on contributing and reporting bugs that we ask you to review.
Following these guidelines helps to communicate that you respect the time of the
contributors managing and developing this open source project. In return, they
should reciprocate that respect in addressing your issue, assessing changes, and
helping you finalize your pull requests. In that spirit of mutual respect, we
endeavor to review incoming issues and pull requests within 10 days, and will
close any lingering issues or pull requests after 60 days of inactivity.

Please note that all of your interactions in the project are subject to our
[Code of Conduct](/CODE_OF_CONDUCT.md). This includes creation of issues or pull
requests, commenting on issues or pull requests, and extends to all interactions
in any real-time space e.g., Slack, Discord, etc.

## Reporting Issues

Before reporting a new issue, please ensure that the issue was not already
reported or fixed by searching through our [issues
list](https://github.com/cisco-ai-defense/pickle-whip/issues).

When creating a new issue, please be sure to include:

- **Clear title**: Descriptive summary of the issue
- **Description**: Detailed explanation of the problem
- **Steps to reproduce**: How to trigger the issue
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, Rust version, pickle-whip version
- **Test case**: Minimal code example (if applicable)

**Example Issue Template:**
```markdown
### Description
Generation fails when using protocol V4 with large opcode ranges.

### Steps to Reproduce
1. Run: `pickle-whip --protocol 4 --min-opcodes 500 --max-opcodes 1000 output.pkl`
2. Observe error message

### Expected Behavior
Should generate a valid pickle file.

### Actual Behavior
Fails with error: "FRAME size calculation overflow"

### Environment
- OS: macOS 14.0
- Rust: 1.75.0
- pickle-whip: 0.1.0
```

**If you discover a security bug, please do not report it through GitHub.
Instead, please see security procedures in [SECURITY.md](/SECURITY.md).**

## Suggesting New Features

We welcome feature suggestions! Before creating a feature request:

1. **Check existing issues**: Search for similar feature requests
2. **Consider scope**: Ensure the feature aligns with project goals
3. **Provide use case**: Explain why this feature would be valuable

**Feature Request Template:**
```markdown
### Feature Description
Add support for custom mutator plugins.

### Use Case
Allow users to define domain-specific mutation strategies for targeted fuzzing.

### Proposed Implementation
- Plugin API using dynamic loading
- Trait-based mutator interface
- Configuration via TOML file

### Alternatives Considered
- Static mutator registration (less flexible)
- Python-only mutators (performance concerns)
```

Feature requests can be submitted via [GitHub Issues](https://github.com/cisco-ai-defense/pickle-whip/issues) with the "enhancement" label.

## Sending Pull Requests

Before sending a new pull request, take a look at existing pull requests and
issues to see if the proposed change or fix has been discussed in the past, or
if the change was already implemented but not yet released.

We expect new pull requests to include tests for any affected behavior. This project
follows [Semantic Versioning 2.0.0](https://semver.org/), and we may reserve breaking
changes until the next major version release.

### Versioning Policy

**Semantic Versioning (SemVer):**
- **MAJOR** (x.0.0): Incompatible API changes, breaking changes
- **MINOR** (0.x.0): New features, backwards-compatible
- **PATCH** (0.0.x): Bug fixes, backwards-compatible

**Breaking Changes:**
- Changes that require users to modify their code
- Removal of public APIs or features
- Changes to CLI behavior or flags
- Changes to output formats

**Pre-1.0 Versions:**
During the 0.x.x phase, minor versions may include breaking changes as the API stabilizes.
Once 1.0.0 is released, breaking changes will only occur in major version updates.

### Pull Request Guidelines

**Before submitting:**
- [ ] Create an issue first (for non-trivial changes)
- [ ] Fork the repository and create a feature branch
- [ ] Write tests for your changes
- [ ] Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
- [ ] Update documentation if needed
- [ ] Add entry to CHANGELOG.md (if applicable)

**Pull Request Description Template:**
```markdown
## Description
Brief summary of changes.

## Related Issue
Fixes #123

## Changes Made
- Added X feature
- Fixed Y bug
- Updated Z documentation

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests passing

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] No breaking changes (or documented if necessary)
```

### Commit Message Guidelines

We follow conventional commit format for clear history:

**Format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```bash
# Good commit messages
feat(generator): add support for custom mutators
fix(opcodes): correct FRAME size calculation for large pickles
docs(readme): update installation instructions
test(generator): add tests for protocol V5

# Bad commit messages
fixed stuff
update
WIP
```

**Best Practices:**
- Use present tense ("add feature" not "added feature")
- Keep subject line under 72 characters
- Provide detailed body for non-trivial changes
- Reference issue numbers in footer (e.g., "Fixes #123")

## Development Setup

### Prerequisites

- Rust 1.70 or later

### Building from Source

```bash
git clone https://github.com/cisco-ai-defense/pickle-whip
cd pickle-whip
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Coverage

We maintain >70% code coverage. Before submitting a PR, please ensure your changes include tests:

```bash
# Install tarpaulin (first time only)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View the report
open coverage/tarpaulin-report.html  # macOS
xdg-open coverage/tarpaulin-report.html  # Linux
```

**Coverage Requirements:**
- Overall coverage should remain >70%
- New code should have >80% coverage
- All public APIs must be tested
- Include both unit tests and integration tests where appropriate

See [TESTING.md](TESTING.md) for detailed testing guidelines.

### Writing Tests

All new features and bug fixes should include tests:

**Unit Tests** - Add to the bottom of source files:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Test implementation
    }
}
```

**Integration Tests** - Add to `tests/` directory:
```rust
use pickle_whip::{Generator, Version};

#[test]
fn test_integration_scenario() {
    let mut gen = Generator::new(Version::V3);
    let result = gen.generate();
    assert!(result.is_ok());
}
```

### Code Style

We follow standard Rust formatting conventions:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check everything before submitting
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Participating in Discussions

We encourage community participation beyond code contributions:

### GitHub Discussions

Use [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-whip/discussions) for:
- **Questions**: Ask about usage, features, or implementation
- **Ideas**: Share thoughts on future directions
- **Show and Tell**: Share projects using pickle-whip
- **General**: Community chat and collaboration

### Discussion Guidelines

- **Be respectful**: Follow our [Code of Conduct](CODE_OF_CONDUCT.md)
- **Search first**: Check if your question has been answered
- **Be specific**: Provide context and examples
- **Stay on topic**: Keep discussions focused
- **Help others**: Share your knowledge and experience

### Issue Triage

Help maintain the project by:
- Reproducing reported bugs
- Adding missing information to issues
- Suggesting labels or categorization
- Closing resolved or duplicate issues

## Other Ways to Contribute

We welcome anyone that wants to contribute to `pickle-whip` to triage and
reply to open issues to help troubleshoot and fix existing bugs. Here is what
you can do:

- **Issue Triage**: Help ensure that existing issues follow the recommendations from the
  _[Reporting Issues](#reporting-issues)_ section, providing feedback to the
  issue's author on what might be missing.
- **Documentation**: Review and update the existing content of our documentation with up-to-date
  instructions and code samples.
- **Code Review**: Review existing pull requests, and test patches against real existing
  applications that use `pickle-whip`.
- **Testing**: Write a test, or add a missing test case to an existing test.
- **Examples**: Create example projects or tutorials showing pickle-whip usage.
- **Benchmarking**: Run performance tests and report results.
- **Bug Hunting**: Use pickle-whip to find bugs in pickle implementations and report findings.

## Review Process

### Timeline

- **Initial Response**: Within 10 days
- **Review Cycles**: Typically 3-7 days between reviews
- **Stale Issues/PRs**: Closed after 60 days of inactivity

### What to Expect

Maintainers will:
1. Review your contribution for technical correctness
2. Check adherence to coding standards
3. Verify tests and documentation
4. Provide constructive feedback
5. Merge when all requirements are met

### Getting Help

If you need assistance:
- Comment on your PR or issue
- Ask in [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-whip/discussions)
- Be patient - maintainers are volunteers

Thanks again for your interest in contributing to `pickle-whip`!

:heart:
