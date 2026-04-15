# up

`tx3up` is the installer and version manager for the [Tx3](https://github.com/tx3-lang) toolchain. It downloads the tools described by a channel manifest, keeps them up to date, and wires them into your `PATH`.

## Installation

Install the latest release with the bootstrap script:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tx3-lang/up/releases/latest/download/tx3up-installer.sh | sh
```

Once `tx3up` is on your `PATH`, it manages itself and the rest of the toolchain.

## Usage

```sh
tx3up                      # install or update everything on the active channel
tx3up install              # same as above
tx3up install --release v0.8.0  # pin the manifest to a specific toolchain release
tx3up check                # report available updates without installing
tx3up use stable           # switch the default channel (stable, beta, nightly, …)
tx3up show                 # list installed tools and their versions
```

Global flags (also available as env vars):

| Flag | Env var | Purpose |
| --- | --- | --- |
| `--root-dir` | `TX3_ROOT` | Installation root (default: `~/.tx3`) |
| `--channel` | `TX3_CHANNEL` | Override the active channel for one command |
| `--github-token` | `GITHUB_TOKEN` | Authenticated GitHub requests (higher rate limits) |

## How it works

`tx3up` is a thin orchestrator around **channel manifests** published as assets on releases of [`tx3-lang/toolchain`](https://github.com/tx3-lang/toolchain).

1. **Channel manifest.** For the active channel, `tx3up` downloads `manifest-<channel>.json` from the latest (or pinned) toolchain release. The manifest lists every tool in the toolchain with its source repo and required semver.
2. **Version check.** Each installed binary is invoked with `--version` and compared against the manifest's requirement. Tools that are missing or out of date become update candidates.
3. **Install.** For each update, `tx3up` queries the tool's own GitHub releases, picks the newest release matching the manifest's `VersionReq`, downloads the asset for the current `os`/`arch`, and extracts the binary into the channel's `bin/` directory.
4. **PATH wiring.** On first install, `tx3up` appends the channel `bin/` to the user's shell profile so the tools are available in new shells.

### On-disk layout

```
~/.tx3/
├── default -> stable          # symlink to the active channel
├── stable/
│   ├── bin/                   # installed tool binaries
│   ├── manifest.json          # cached channel manifest
│   └── updates.json           # cached update state
├── beta/
└── nightly/
```

Channels are fully isolated — switching with `tx3up use <channel>` just repoints the `default` symlink, so multiple channels can coexist without reinstalling.

### Source layout

- `src/main.rs` — CLI entrypoint, global config, channel/path resolution.
- `src/cmds/` — one module per subcommand (`install`, `check`, `use`, `show`).
- `src/manifest.rs` — manifest fetching, caching, and staleness checks.
- `src/updates.rs` — comparing installed versions against manifest requirements.
- `src/perm_path.rs` — adding the channel `bin/` directory to the user's shell profile.
- `src/bin.rs` — binary extraction helpers (tar.gz / tar.xz).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the development workflow and [`tests/e2e/README.md`](tests/e2e/README.md) for the end-to-end test harness.
