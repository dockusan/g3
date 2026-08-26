# g3

Desktop three-way merge for git conflicts.

## Run from a conflicted repo

Put the launcher on your `PATH` once:

```bash
export PATH="/path/to/marlin/bin:$PATH"
```

Then, from any git repo with merge conflicts:

```bash
g3
```

Or pass a path:

```bash
g3 /path/to/conflicted-repo
```

## Development

```bash
pnpm install
./bin/g3 /path/to/conflicted-repo
```

If no built binary exists, this starts `pnpm tauri dev` and points the window at that repo. The first launch compiles Rust and can take a while.

## Production build

```bash
pnpm tauri build
g3 /path/to/conflicted-repo
```

The `g3` launcher uses the release binary when it exists, otherwise falls back to `tauri dev`.

## Tests

```bash
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
