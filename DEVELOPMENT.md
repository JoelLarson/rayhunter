# Development

**Prerequisites:** Rust (via [rustup](https://rustup.rs)) and Node.js/npm.

## Getting started

First-time setup — installs the Rust ARM target and all npm dependencies:

```
make install-deps
```

Build everything and launch the GUI installer in dev mode:

```
make run-installer
```

## All targets

| Target | What it does |
|---|---|
| `make install-deps` | Install Rust ARM target and all npm dependencies |
| `make build-daemon-web` | Build the daemon's embedded web UI |
| `make build-firmware` | Cross-compile rootshell and daemon for the device |
| `make run-installer` | Launch the GUI installer in dev mode (builds all prerequisites) |
| `make package-installer` | Bundle a distributable release of the GUI installer |
| `make clean` | Remove all build artifacts |

Run `make` with no arguments to print this reference.
