# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview / usage

See `README.md`. The CLI surface is defined in `src/cli.rs` (clap derive); the `ArgGroup` there enforces that exactly one of `--nfc`/`--nfd` is required.

## Toolchain and commands

- Rust version: pinned in `mise.toml` (`mise install` if missing).
- Lint/format/test commands and the OS × target matrix: `.github/workflows/ci.yml` is the source of truth — mirror those locally rather than maintaining a duplicate list here.
- Tests are colocated in `src/main.rs` under `mod tests`. Run a single test with `cargo test <fn_name>` (e.g. the function names in that module).

## Architecture (non-obvious points)

Modules under `src/` (`cli.rs`, `norm.rs`, `rename.rs`, `main.rs`) are small enough that reading them is faster than reading a description. The points below are the things that are *not* obvious from the code alone.

### macOS APFS same-inode quirk (load-bearing)

On APFS, NFC and NFD filenames resolve to the **same inode**, so a direct `rename(nfd, nfc)` is a no-op. `execute_op` in `src/rename.rs` detects this via `same_inode` and routes through a `ucmvtmpN` intermediate (same workaround `convmv` uses). `check_op` correspondingly only flags a destination conflict when inodes differ, so legitimate macOS renames aren't rejected.

The integration test `execute_does_not_overwrite_existing_file` in `src/main.rs` deliberately asserts **different things** under `#[cfg(target_os = "macos")]` vs `#[cfg(not(...))]`. Preserve that split when editing rename logic; CI runs both Linux and macOS targets (see `.github/workflows/ci.yml`) so divergence will be caught.

### Walk order

`collect_ops` uses `WalkDir::contents_first(true)` on purpose: with `-r`, directory names themselves may be renamed, so children must be processed before their parent — otherwise the parent path changes mid-walk and queued child paths go stale.

### Batch error handling

`run()` in `src/main.rs` prints per-op errors to stderr and keeps going rather than aborting the batch. Follow that pattern for new failure modes; don't propagate a single rename failure up through `?`.
