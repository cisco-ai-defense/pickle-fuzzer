# How to Contribute

Thanks for your interest in contributing to `cisco-ai-defense-pickle-fuzzer`! Here are a few
general guidelines on contributing and reporting bugs that we ask you to review.
Following these guidelines helps to communicate that you respect the time of the
contributors managing and developing this open source project. In return, they
should reciprocate that respect in addressing your issue, assessing changes, and
helping you finalize your pull requests. In that spirit of mutual respect, we
endeavor to review incoming issues and pull requests within 10 days, and will
close any lingering issues or pull requests after 60 days of inactivity.

Please note that all of your interactions in the project are subject to our
[Code of Conduct](CODE_OF_CONDUCT.md). This includes creation of issues or pull
requests, commenting on issues or pull requests, and extends to all interactions
in any real-time space e.g., Slack, Discord, etc.

## Reporting Issues

Before creating an issue, please:
- Search [existing issues](https://github.com/cisco-ai-defense/pickle-fuzzer/issues) to avoid duplicates
- Provide clear reproduction steps
- Include environment details (OS, Rust version, package version)

**Security Issues**: Please report security vulnerabilities via [SECURITY.md](SECURITY.md), not through GitHub issues.

## Feature Requests

We welcome feature suggestions! Please:
- Check for existing feature requests first
- Clearly describe the use case and benefits
- Consider if it aligns with the project's goals

## Pull Requests

Before submitting a pull request:

1. **Check for existing work**: Search issues and PRs to avoid duplicates
2. **Create an issue first**: For non-trivial changes, discuss the approach
3. **Follow the development guide**: See [DEVELOPING.md](DEVELOPING.md) for setup and workflows
4. **Include tests**: All changes must include appropriate tests
5. **Run checks**: Ensure `cargo fmt && cargo clippy -- -D warnings && cargo test` passes
6. **Update docs**: Update relevant documentation

### Pull Request Checklist

- [ ] Tests added/updated and passing
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation updated
- [ ] Commit messages are clear and descriptive

### Versioning

This project follows [Semantic Versioning](https://semver.org/). During the 0.x phase, minor versions may include breaking changes.

## Development

For detailed development instructions, see:
- **[DEVELOPING.md](DEVELOPING.md)** - Setup, workflows, and best practices
- **[TESTING.md](TESTING.md)** - Testing guidelines and requirements

### Quick Start

```bash
git clone https://github.com/cisco-ai-defense/pickle-fuzzer
cd pickle-fuzzer
cargo build
cargo test
```

## Community

- **Discussions**: Use [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-fuzzer/discussions) for questions and ideas
- **Issues**: Report bugs via [GitHub Issues](https://github.com/cisco-ai-defense/pickle-fuzzer/issues)
- **Review**: Help by reviewing PRs and testing changes

## Getting Help

If you need assistance:
- Check existing documentation and issues
- Ask in [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-fuzzer/discussions)
- Be patient - maintainers are volunteers

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

Thank you for contributing! 🎉
