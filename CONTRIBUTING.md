# Contributing to diffscape

Thank you for your interest in contributing to diffscape! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.85 or higher
- Git
- A text editor or IDE of your choice

### Setting up the Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/yourusername/diffscape.git
   cd diffscape
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run the tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make sure all tests pass:
   ```bash
   cargo test
   ```

### Making Changes

1. Write your code following the project's style guidelines
2. Add tests for new functionality
3. Update documentation if necessary
4. Run the formatting and linting tools:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```

### Testing

- Run all tests: `cargo test`
- Run tests with coverage: `cargo tarpaulin --all-features`
- Test different scenarios manually with various git repositories

### Submitting Changes

1. Commit your changes with clear, descriptive commit messages
2. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
3. Create a Pull Request on GitHub

## Pull Request Guidelines

### Before Submitting

- [ ] All tests pass
- [ ] Code is formatted with `cargo fmt`
- [ ] No Clippy warnings
- [ ] Documentation is updated if necessary
- [ ] CHANGELOG.md is updated (if applicable)

### PR Description

Please include:
- A clear description of what the change does
- Why the change is needed
- Any breaking changes
- Screenshots or terminal recordings for UI changes

## Code Style

### Rust Code

- Follow standard Rust formatting (`cargo fmt`)
- Use `cargo clippy` and address all warnings
- Write clear, descriptive variable and function names
- Add comments for complex logic
- Write tests for new functionality

### Commit Messages

Use clear, descriptive commit messages:
- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 50 characters
- Reference issues and PRs when applicable

Examples:
```
Add syntax highlighting for TypeScript files
Fix alignment issue in unified diff mode
Update README with installation instructions
```

### Documentation

- Update README.md for user-facing changes
- Add rustdoc comments for public APIs
- Update CHANGELOG.md for notable changes

## Issue Guidelines

### Bug Reports

When reporting bugs, please include:
- Operating system and version
- Rust version
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages (if any)

### Feature Requests

For feature requests, please describe:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered
- Whether you're willing to implement it

## Code of Conduct

### Our Pledge

We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project maintainers. All complaints will be reviewed and investigated and will result in a response that is deemed necessary and appropriate to the circumstances.

## Getting Help

If you need help with development:
- Check the existing issues and discussions
- Create a new issue with the "question" label
- Join our discussions on GitHub

## Recognition

Contributors will be recognized in:
- The project's README
- Release notes for significant contributions
- GitHub's contributor graph

Thank you for contributing to diffscape!