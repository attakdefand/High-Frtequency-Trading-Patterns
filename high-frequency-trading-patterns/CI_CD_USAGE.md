# CI/CD Usage Guide

This document explains how to use and maintain the CI/CD pipeline for the high-frequency trading patterns project.

## Triggering Workflows

### CI Workflow
The CI workflow runs automatically on:
- Push to main or master branches
- Pull requests to main or master branches

### CD Workflow
The CD workflow runs automatically when:
- A tag matching the pattern `v*.*.*` is pushed

### Quality Workflow
The Quality workflow runs automatically on:
- Push to main or master branches
- Pull requests to main or master branches

## Creating a Release

To create a new release:

1. Ensure all changes are merged to the main branch
2. Create and push a new tag:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```
3. The CD workflow will automatically:
   - Build release binaries
   - Run all tests
   - Create a draft GitHub release

## Adding New Patterns

When adding new patterns to the workspace:

1. Add the new crate to the workspace in `Cargo.toml`
2. The CI workflow will automatically include it in builds and tests
3. No changes to workflow files are needed

## Customizing Checks

### Adding New Lints
To add new Clippy lints:
1. Modify the Clippy command in the workflow files
2. Ensure the lint is available in the Rust toolchain

### Adding New Tests
New tests in any crate will automatically be picked up by the CI workflow.

### Adding Security Checks
To add additional security checks:
1. Add new steps to the quality workflow
2. Ensure any new tools are installed in the workflow

## Debugging Failures

### Accessing Logs
All workflow logs are available in the GitHub Actions tab of the repository.

### Re-running Workflows
Failed workflows can be re-run from the GitHub Actions interface.

### Testing Locally
Most CI checks can be run locally:
```bash
# Install cargo-make
cargo install cargo-make

# Run all CI checks
cargo make ci
```

## Environment Variables

The workflows use these environment variables:
- `CARGO_TERM_COLOR=always` - Ensures colored output in logs

Additional environment variables can be added to the workflow files as needed.

## Caching Strategy

The workflows use GitHub Actions caching for:
- Cargo registry (`~/.cargo/registry`)
- Cargo git dependencies (`~/.cargo/git`)
- Build artifacts (`target` directory)

This significantly speeds up subsequent runs.

## Permissions

The CD workflow requires these permissions:
- `contents: write` - To create GitHub releases

These are configured in the workflow file.

## Notifications

Workflow failures will trigger notifications based on your GitHub settings.