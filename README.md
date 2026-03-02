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
