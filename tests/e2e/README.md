# tx3up E2E tests

End-to-end tests for `tx3up`. They drive a real `tx3up` binary against a temporary `TX3_ROOT` and assert the resulting toolchain layout.

## Tests

- `fresh_install.sh` — runs `tx3up` against an empty root and verifies the channel directory, `manifest.json`, and at least one executable in `bin/` are created.
- `update_install.sh` — installs once, runs `tx3up` again, and checks that the manifest is refreshed, no binaries are lost, and a third run is idempotent.

Both scripts read `TX3_CHANNEL` from the environment (default: `stable`). They allocate their own `TX3_ROOT` via `mktemp -d` and clean it up on exit, so they will not touch your real `~/.tx3`.

## Running locally

From the repo root:

```sh
cargo build --release
./tests/e2e/fresh_install.sh
./tests/e2e/update_install.sh
```

To exercise a non-default channel:

```sh
TX3_CHANNEL=nightly ./tests/e2e/fresh_install.sh
```

Requirements: a working Rust toolchain and `jq` (the scripts fall back to a basic JSON check if `jq` is missing, but installing it is recommended). The scripts hit the real `tx3-lang/toolchain` GitHub releases, so the host needs network access. If you run into GitHub rate limits, export `GITHUB_TOKEN` before running.

## How CI uses these tests

`.github/workflows/e2e.yml` runs both scripts on every pull request to `main` and on manual `workflow_dispatch`. The workflow:

1. Checks out the PR.
2. Builds `tx3up` from source with `cargo build --release` on each runner.
3. Runs `fresh_install.sh` and `update_install.sh` against the freshly-built binary.

The matrix covers `ubuntu-latest`, `ubuntu-24.04-arm`, and `macos-latest`, with each script as a separate cell so failures are isolated. `workflow_dispatch` accepts a `test_channel` input (`stable`/`nightly`/`beta`); PR runs always use `stable`.

Because CI builds from the PR's source, these tests gate the **current** code — not a previously-released binary.
