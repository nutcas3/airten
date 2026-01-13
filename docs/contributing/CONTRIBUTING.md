# Contributing to AirTen

Thank you for your interest in contributing to AirTen!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/airten.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Submit a pull request

## Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
make install-tools

# Build and test
make all
```

## Code Style

- Follow Rust API Guidelines
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Write documentation for public APIs
- Add tests for new functionality

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure CI passes
4. Request review from maintainers

## Reporting Issues

- Use GitHub Issues
- Include reproduction steps
- Provide system information
- Attach relevant logs

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
