# Build Matrix

CollectionLoom is portable-first. This repository supports three distribution modes:

- **Source build**: developer-friendly, fully reproducible from GitHub.
- **Portable build**: the normal app experience for end users, bundled with any third-party tools that have official release artifacts.
- **Commercial build**: a separately distributed paid portable binary that is not published in GitHub Releases.

## Platform matrix

| Platform | Source build | Portable build | Commercial binary | Notes |
|----------|--------------|----------------|-------------------|-------|
| macOS | `npm run tauri:build` | `npm run build:portable` | external distribution | Portable app launches directly; AVML is bundled when an official release artifact is available, while MRS remains source-specific/manual |
| Windows | `npm run tauri:build` | `npm run build:portable` | external distribution | Portable app launches directly; AVML and WinPmem are bundled when official release artifacts are available |
| Linux | `npm run tauri:build` | `npm run build:portable` | external distribution | Portable app launches directly; AVML is bundled when an official release artifact is available, while LiME remains source-specific/manual |

## Tool availability

| Tool | Status | Distribution guidance |
|------|--------|-----------------------|
| `avml` | downloadable | Include in source/portable builds when an official release artifact is available for the current platform |
| `winpmem` | downloadable | Include in Windows builds from the official release artifact |
| `lime` | source-specific | Keep as an explicit manual or environment-specific staging step |
| `mrs` | source-specific | Keep as a macOS-specific staging step rather than a generic binary download |

## Recommended workflows

### Source build

Use this when developing, testing, or auditing CollectionLoom. The helper downloads every official artifact that exists for the current platform into `tools/`, then mirrors that folder into the app bundle:

```bash
npm install
npm run tauri:build
```

### Portable build

Use this when preparing the normal no-install field kit or validating the shipped end-user experience:

```bash
npm install
npm run build:portable
```

### Commercial build

Use this when producing the paid portable binary for external distribution:

```bash
npm install
npm run build:commercial
```

## CI intent

Public CI should verify source builds, portable packaging, and automated tests without mixing the commercial release channel into GitHub Releases. Commercial packaging should be published through the separate sales/distribution pipeline.
