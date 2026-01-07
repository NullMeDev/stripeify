# Contributing to Stripeify

First off, thank you for considering contributing to Stripeify! It's people like you that make Stripeify such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* **Use a clear and descriptive title**
* **Describe the exact steps which reproduce the problem**
* **Provide specific examples to demonstrate the steps**
* **Describe the behavior you observed after following the steps**
* **Explain which behavior you expected to see instead and why**
* **Include screenshots if possible**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* **Use a clear and descriptive title**
* **Provide a step-by-step description of the suggested enhancement**
* **Provide specific examples to demonstrate the steps**
* **Describe the current behavior and explain which behavior you expected to see instead**
* **Explain why this enhancement would be useful**

### Pull Requests

* Fill in the required template
* Do not include issue numbers in the PR title
* Follow the Rust style guide
* Include thoughtfully-worded, well-structured tests
* Document new code
* End all files with a newline

## Development Setup

### Prerequisites

* Rust 1.70 or higher
* ChromeDriver
* Git

### Setting Up Your Development Environment

1. Fork the repo
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/stripeify.git
   cd stripeify
   ```

3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/NullMeDev/stripeify.git
   ```

4. Create a branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. Build the project:
   ```bash
   cargo build
   ```

6. Run tests:
   ```bash
   cargo test
   ```

## Style Guidelines

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line

Example:
```
Add proxy rotation feature

- Implement proxy pool management
- Add automatic failover
- Update documentation

Fixes #123
```

### Rust Style Guide

* Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
* Run `cargo fmt` before committing
* Run `cargo clippy` and fix all warnings
* Write documentation for public APIs
* Add tests for new functionality

### Documentation Style Guide

* Use Markdown for documentation
* Keep line length to 80 characters when possible
* Use code blocks with language specification
* Include examples where appropriate

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run with coverage
cargo tarpaulin
```

### Writing Tests

* Write unit tests for individual functions
* Write integration tests for features
* Use descriptive test names
* Test edge cases
* Mock external dependencies

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_validation() {
        let card = "4532015112830366";
        assert!(validate_card(card).is_ok());
    }

    #[test]
    fn test_invalid_card() {
        let card = "1234567890123456";
        assert!(validate_card(card).is_err());
    }
}
```

## Project Structure

```
stripeify/
├── src/                    # Source code
│   ├── main.rs            # Entry point
│   ├── checker.rs         # Core logic
│   └── ...
├── tests/                  # Integration tests
├── docs/                   # Documentation
├── examples/               # Example configs
└── scripts/                # Utility scripts
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag
4. Push to GitHub
5. Create a GitHub release

## Questions?

Feel free to open an issue with your question or reach out to the maintainers:

* Email: support@nullme.dev
* Telegram: [@MissNullMe](https://t.me/MissNullMe)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:
* The README.md file
* Release notes
* The project website (when available)

Thank you for contributing to Stripeify! 🎉
