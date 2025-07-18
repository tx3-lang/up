# up
The Tx3 toolchain installer

## Running E2E Tests Locally

The project includes end-to-end tests that verify the `tx3up` installer functionality. These tests can be run locally to ensure the installer works correctly in your environment.

### Prerequisites

1. **Build the binary**: First, build the release version of `tx3up`:
   ```bash
   cargo build --release
   ```

2. **Install jq** (optional, for enhanced JSON validation):
   ```bash
   # On Ubuntu/Debian
   sudo apt-get install jq
   
   # On macOS
   brew install jq
   ```

### Available Tests

- **`fresh_install.sh`**: Tests a clean installation of the toolchain
- **`update_install.sh`**: Tests updating an existing installation

### Running Tests

#### Run Individual Tests

```bash
# Run fresh install test
./tests/e2e/fresh_install.sh

# Run update install test
./tests/e2e/update_install.sh
```

#### Run All Tests

```bash
# Run all e2e tests
for test in tests/e2e/*.sh; do
    echo "Running $test..."
    "$test"
done
```

#### Custom Test Configuration

You can customize the test behavior using environment variables:

```bash
# Use a specific installation directory
export TX3_ROOT_DIR="/tmp/my_tx3_test"
./tests/e2e/fresh_install.sh

# Test with a different channel (default is "stable")
export TX3_CHANNEL="nightly"
./tests/e2e/fresh_install.sh

# Combine both parameters
export TX3_ROOT_DIR="/tmp/my_tx3_test"
export TX3_CHANNEL="beta"
./tests/e2e/update_install.sh

# The test will automatically clean up after itself
```

### Test Output

Each test provides colored output showing:
- ✅ Green: Successful operations
- ❌ Red: Failed operations  
- ⚠️ Yellow: Warnings or informational messages

The tests will:
1. Create a temporary installation directory
2. Run the installer with the test configuration
3. Verify the installation artifacts
4. Clean up temporary files automatically

### CI/CD

The E2E tests can be run manually in GitHub Actions using the "E2E Tests" workflow:

**Manual Workflow Trigger**
- Navigate to Actions → E2E Tests → Run workflow
- Specify the GitHub release tag to test (e.g., `v0.4.2`)
- Choose the TX3 channel to test (`stable`, `nightly`, or `beta`)

**How it works:**

**Stage 1: Download Binaries** (4 jobs)
- Downloads release binaries from GitHub releases for each platform:
  - Ubuntu Intel (x86_64-unknown-linux-gnu)
  - Ubuntu ARM (aarch64-unknown-linux-gnu)  
  - macOS Intel (x86_64-apple-darwin)
  - macOS ARM (aarch64-apple-darwin)

**Stage 2: Run Tests** (8 jobs)
- Downloads the appropriate binary for each platform
- Runs both `fresh_install` and `update_install` tests
- Tests against the specified channel

This approach tests actual released binaries rather than building from source, ensuring the published artifacts work correctly.

Check the `.github/workflows/e2e.yml` file for the complete workflow configuration.
