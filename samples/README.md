# Sample Files for CollectionLoom

Real files used in integration tests, documentation screenshots, and hash verification demos.

| File | Size | Purpose |
|------|------|---------|
| `verify_me.txt` | 21 B | Hash verification target |
| `expected.sha256` | — | Known SHA-256 for `verify_me.txt` |
| `source_disk.img` | 10 MB | Synthetic disk image for imaging tests |
| `case_notes.txt` | — | Sample investigator notes |

## Verify hash

```bash
shasum -a 256 verify_me.txt
# Should match the hash in expected.sha256
```

## Run tests

```bash
cd packages/collectionloom/src-tauri
cargo test forensic_test
```

## Regenerate screenshots

```bash
node scripts/prepare-screenshot-data.mjs
cd packages/collectionloom && VITE_FIXTURE_MODE=1 npm run build
node scripts/capture-screenshots.mjs
```

Fixture data in `packages/collectionloom/public/fixtures/screenshot-data.json` is generated from these files — SHA-256 values are computed from actual bytes on disk.
