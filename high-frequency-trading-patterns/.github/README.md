# CI/CD Documentation

This project uses GitHub Actions for continuous integration and continuous deployment.

## Workflows

### CI (Continuous Integration)
- **File**: [.github/workflows/ci.yml](workflows/ci.yml)
- **Trigger**: Push or pull request to main/master branches
- **Steps**:
  1. Install Rust toolchain
  2. Cache dependencies
  3. Build workspace
  4. Run tests
  5. Check code formatting
  6. Run Clippy linter

### CD (Continuous Deployment)
- **File**: [.github/workflows/cd.yml](workflows/cd.yml)
- **Trigger**: Tagged releases (v*.*.*)
- **Steps**:
  1. Install Rust toolchain
  2. Cache dependencies
  3. Build release binaries
  4. Run tests
  5. Create GitHub release

### Code Quality
- **File**: [.github/workflows/quality.yml](workflows/quality.yml)
- **Trigger**: Push or pull request to main/master branches
- **Steps**:
  1. Install Rust toolchain
  2. Cache dependencies
  3. Check code formatting
  4. Run Clippy linter
  5. Run security audit

## Local Development

### Using cargo make

This project includes a [Makefile.toml](../Makefile.toml) for cargo make:

```bash
# Install cargo-make
cargo install cargo-make

# Run all CI checks locally
cargo make ci

# Build release version
cargo make release
```

### Manual Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Run security audit
cargo audit
```

## Configuration Files

- [deny.toml](../deny.toml) - Configuration for cargo-deny (license and security checks)
- [Makefile.toml](../Makefile.toml) - Configuration for cargo-make