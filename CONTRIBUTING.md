# Contributing to tx3up

Thanks for your interest in contributing!

## Before pushing

Run the end-to-end install tests locally before pushing any change that could affect the install flow:

```sh
cargo build --release
./tests/e2e/fresh_install.sh
./tests/e2e/update_install.sh
```

These are the same scripts CI runs against your pull request. Catching failures locally is much faster than waiting for the matrix in `.github/workflows/e2e.yml` to complete. See [`tests/e2e/README.md`](tests/e2e/README.md) for details on what each script checks and how to run them against a non-default channel.
