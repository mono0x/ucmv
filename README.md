# ucmv

Rename files by converting Unicode normalization form (NFC/NFD).

On macOS, the Foundation framework normalizes filenames to NFD when creating files through Cocoa APIs, which can cause issues when exchanging files with Linux or Windows systems that expect NFC. APFS itself stores filenames as-is without normalization, but filenames created via the Foundation framework end up in NFD. `ucmv` converts filenames between NFC and NFD.

## Usage

```
ucmv [OPTIONS] <--nfc|--nfd> [PATHS]...

Options:
      --nfc        Convert NFD filenames to NFC
      --nfd        Convert NFC filenames to NFD
      --notest     Actually rename files (default: dry-run)
  -r, --recursive  Process directories recursively
  -h, --help       Print help
  -V, --version    Print version
```

By default, `ucmv` runs in dry-run mode and only prints what would be renamed. Pass `--notest` to apply the changes.

## Examples

Preview which files would be renamed:

```sh
ucmv --nfc ./photos
```

Rename files recursively:

```sh
ucmv --nfc --notest -r ./photos
```

## Development

Install the tools and Git hook with `mise install` and `hk install`. Run
`hk check --all` to check the repository, or `hk fix` to format changed files.
Run `cargo clippy --all-targets --all-features -- -D warnings` separately.

## Releasing

Run the Bump version workflow manually on `main`. It updates `Cargo.toml` and
`Cargo.lock` to `0.YYYYMMDD.N` and opens a release pull request with auto-merge
enabled. The date uses Asia/Tokyo, and `N` is the commit count before the version
bump. Once CI passes and the pull request is merged, the Tag workflow creates the
`v0.YYYYMMDD.N` tag, which triggers the Release workflow. Both workflows act as a
GitHub App (`RELEASE_APP_CLIENT_ID` variable and `RELEASE_APP_PRIVATE_KEY` secret)
with Contents and Pull requests read and write permission, because events caused
by `GITHUB_TOKEN` do not trigger workflows. GoReleaser builds binaries for Linux x86_64 and
macOS x86_64/ARM64 in a reusable workflow without write access to the repository,
and a separate job publishes them with checksums and GitHub-generated release notes.

To build release artifacts locally without publishing (requires macOS and Xcode
Command Line Tools):

```sh
MISE_ENV=release mise install
MISE_ENV=release mise exec -- goreleaser release --snapshot --clean
```

Artifacts are written to `dist/` using GoReleaser's default file names.

Release binaries are [reproducible](https://reproducible-builds.org/). To verify
a release, check out its tag and build it as above; the SHA-256 digests of the
binaries in `dist/` should match the entries in the release's `checksums.txt`.

Release binaries also carry [SLSA Build Level 3](https://slsa.dev/spec/v1.0/levels#build-l3)
provenance. Verify a downloaded binary with:

```sh
gh attestation verify <file> --repo mono0x/ucmv \
  --signer-workflow mono0x/ucmv/.github/workflows/release-build.yml
```
