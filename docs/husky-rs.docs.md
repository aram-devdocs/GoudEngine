# husky-rs

[![CI](https://github.com/pplmx/husky-rs/workflows/CI/badge.svg)](https://github.com/pplmx/husky-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/husky-rs.svg)](https://crates.io/crates/husky-rs)
[![Documentation](https://docs.rs/husky-rs/badge.svg)](https://docs.rs/husky-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

`husky-rs` is a Git hooks management tool for Rust projects, inspired by Husky.

## Features

- **Easy setup and configuration**
- **Automatic installation of Git hooks**
- **Support for all Git hooks**
- **Cross-platform compatibility** (Unix-like systems and Windows)

## Quick Start

1. Adding `husky-rs` to your project:

   You have several options:

   ```sh
   # Option 1: Add as a Regular Dependency
   cargo add husky-rs

   # Option 2: Add as a Dev Dependency
   cargo add --dev husky-rs

   # Option 3: Use the Main Branch
   cargo add --git https://github.com/pplmx/husky-rs --branch main
   cargo add --dev --git https://github.com/pplmx/husky-rs --branch main
   ```

2. Create hooks directory:

   ```sh
   mkdir -p .husky/hooks
   ```

3. Add a hook (e.g., `pre-commit`):

   ```sh
   echo '#!/bin/sh\necho "Running pre-commit hook"' > .husky/hooks/pre-commit
   ```

4. Install hooks:

   > **Note:** Due to the execution mechanism of `build.rs`, running `cargo clean` is required when installing or updating hooks.

   ```sh
   cargo clean && cargo test
   ```

---

**Tip:** If you add this library to the `[dependencies]` section, both `cargo build` and `cargo test` will work. However, if it's added under `[dev-dependencies]`, only `cargo test` will function as expected.

## Usage

### Supported Git Hooks

`husky-rs` aims to support a wide range of Git hooks, including:

- `pre-commit`
- `prepare-commit-msg`
- `commit-msg`
- `post-commit`
- `pre-push`

For a complete list of supported hooks, refer to the [Git documentation](https://git-scm.com/docs/githooks).

If you encounter any unsupported hooks, please don't hesitate to [open an issue](https://github.com/pplmx/husky-rs/issues).

### Configuration

To skip hook installation:

```sh
NO_HUSKY_HOOKS=1 cargo build
```

## Best Practices

- Keep hooks lightweight to avoid slowing down Git operations
- Use hooks for tasks like running tests, linting code, and validating commit messages
- Non-zero exit status in a hook script will abort the Git operation

## Development

For information on setting up the development environment, running tests, and contributing to the project, please refer to our [Development Guide](docs/development.md).

## Troubleshooting

If you encounter any issues while using `husky-rs`, please check our [Troubleshooting Guide](docs/troubleshooting.md) for common problems and their solutions. If you can't find a solution to your problem, please [open an issue](https://github.com/pplmx/husky-rs/issues) on our GitHub repository.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to submit pull requests, report issues, or suggest improvements.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Changelog

For a detailed history of changes to this project, please refer to our [CHANGELOG.md](CHANGELOG.md).

## Acknowledgments

- Inspired by [cargo-husky](https://github.com/rhysd/cargo-husky)
- Thanks to the Rust community for their amazing tools and libraries