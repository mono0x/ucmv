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

Run the **Release** workflow manually on `main` after its push CI run succeeds.
It uses cargo-release to update `Cargo.toml` and `Cargo.lock` to
`0.YYYYMMDD.<run_number>`, using the initial workflow run date in Asia/Tokyo.
The version commit and tag are pushed atomically to `main`. The workflow uses
`GITHUB_TOKEN`, which must be allowed to push to `main` under the repository's
branch rules. No GitHub App is required.

The reusable build workflow checks out the tag and builds binaries for Linux
x86_64 and macOS x86_64/ARM64 without write access to the repository. A separate
job verifies checksums, uploads the binaries to a draft release, and publishes
it with GitHub-generated release notes after attestation succeeds.

If CI has not succeeded, rerun after it completes. If `main` has advanced before
the version commit is pushed, start a new workflow run. Use **Re-run failed jobs**
to retry a failed build or publication. Artifacts are retained for seven days;
use **Re-run all jobs** to rebuild the existing tag if they have expired.
Retries can replace draft artifacts and leave published releases intact.
Source or workflow fixes require a new run from the corrected `main`.

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
